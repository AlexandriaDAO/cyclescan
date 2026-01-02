# CycleScan Backend Optimization Plan

## Executive Summary

The CycleScan backend is consuming **2.1 trillion cycles per hourly snapshot** - approximately 10x higher than expected. At this burn rate, the canister requires ~50T cycles/day to operate, making it unsustainably expensive. This document details the root cause and proposed optimizations.

## Findings

### Measured Cost

| Metric | Value |
|--------|-------|
| Cycles before `take_snapshot()` | 4,745,377,966,361 |
| Cycles after `take_snapshot()` | 2,632,063,691,434 |
| **Single snapshot cost** | **2,113,314,274,927 (~2.1T)** |
| Tracked canisters | 3,140 |
| Total snapshots stored | 93,688 |
| Snapshots per canister (avg) | ~30 |

### Projected Costs

| Period | Cycle Cost |
|--------|------------|
| Per snapshot | 2.1T |
| Per day (24 snapshots) | ~50T |
| Per week | ~350T |
| Per month | ~1,500T |

## Root Cause Analysis

### Primary Issue: `update_canister_summaries()` (lines 1201-1216)

After collecting snapshots, the code recalculates balance and burn rates for **every canister** by re-reading **all their snapshots** from stable memory:

```rust
fn update_canister_summaries() {
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        let keys: Vec<_> = map.iter().map(|(k, _)| k).collect();

        for key in keys {
            if let Some(mut meta) = map.get(&key) {
                meta.balance = get_latest_balance(&key);           // O(M) - scans all snapshots
                meta.burn_1h = calculate_burn(&key, NANOS_PER_HOUR);   // O(M) - scans all snapshots
                meta.burn_24h = calculate_burn(&key, NANOS_PER_DAY);   // O(M) - scans all snapshots
                meta.burn_7d = calculate_burn(&key, SEVEN_DAYS_NANOS); // O(M) - scans all snapshots
                map.insert(key, meta);
            }
        }
    });
}
```

**Complexity:** O(N × M) where N = canisters (3,140) and M = snapshots per canister (~30)

**Operations per snapshot run:**
- 3,140 × 4 = 12,560 calls to snapshot-scanning functions
- Each scans ~30 snapshots from stable memory
- Total: ~376,800 stable memory range scans
- Plus 3,140 stable memory writes to update canister metadata

### Secondary Issue: `get_latest_balance()` (lines 588-601)

```rust
fn get_latest_balance(key: &PrincipalKey) -> u128 {
    SNAPSHOTS.with(|s| {
        let map = s.borrow();
        // ...
        map.range(start..=end).last()  // <-- Iterates ALL entries to get last
    })
}
```

The `.last()` call on a range iterator is **O(M)**, not O(1). It must traverse all entries in the range.

### Tertiary Issue: `calculate_burn()` (lines 620-674)

```rust
fn calculate_burn(key: &PrincipalKey, window_nanos: u64) -> Option<u128> {
    let snapshots = get_snapshots(key);  // Collects ALL snapshots into Vec
    // ... processes the vec multiple times
}
```

Each call allocates a new `Vec` and collects all snapshots, then iterates multiple times.

### Quaternary Issue: Pruning (lines 1179-1192)

```rust
let to_remove: Vec<_> = map
    .iter()  // Iterates ENTIRE snapshot map (93,688 entries)
    .filter(|(k, _)| k.timestamp < cutoff)
    .map(|(k, _)| k)
    .collect();
```

The snapshot key is ordered by `(canister_id, timestamp)`, so finding old entries requires scanning the **entire map**. Time-based pruning is O(N × M) instead of O(pruned entries).

## Proposed Solution

### Strategy: Inline Metadata Updates

Instead of recalculating all canister metadata in a separate pass, update each canister's metadata **during snapshot insertion** when we already have the new cycles value.

### Implementation

#### 1. Remove `update_canister_summaries()` entirely

#### 2. Modify snapshot insertion to update metadata inline

```rust
// During snapshot insertion:
for (key, result) in results {
    match result {
        Ok(cycles) => {
            // Insert snapshot
            snapshots_map.insert(
                SnapshotKey { canister: key.clone(), timestamp: now },
                CyclesValue(cycles),
            );

            // Update canister metadata inline
            CANISTERS.with(|c| {
                let mut canisters = c.borrow_mut();
                if let Some(mut meta) = canisters.get(&key) {
                    let old_balance = meta.balance;
                    meta.balance = cycles;

                    // Incremental burn calculation using old and new values
                    // instead of rescanning all snapshots
                    update_burns_incremental(&mut meta, old_balance, cycles, now);

                    canisters.insert(key.clone(), meta);
                }
            });

            success += 1;
        }
        Err(_) => failed += 1,
    }
}
```

#### 3. Implement incremental burn calculation

Instead of scanning all snapshots, maintain running calculations:

```rust
fn update_burns_incremental(
    meta: &mut CanisterMeta,
    old_balance: u128,
    new_balance: u128,
    now: u64
) {
    // If balance decreased, that's burn
    if new_balance < old_balance {
        let burn = old_balance - new_balance;
        // Add to rolling burn totals (stored in meta)
        // Decay old burns based on time windows
    }
    // Update the burn_1h, burn_24h, burn_7d fields
}
```

#### 4. Optimize pruning with timestamp-first key (optional, lower priority)

Create a secondary index or change key ordering for efficient time-based queries:

```rust
struct TimestampFirstKey {
    timestamp: u64,
    canister: PrincipalKey,
}
```

This allows pruning old entries with a simple range delete.

## Expected Improvements

| Operation | Current Cost | After Optimization |
|-----------|--------------|-------------------|
| Snapshot insertion | O(1) per canister | O(1) per canister |
| Metadata update | O(N × M) total | O(N) total |
| Stable memory reads | ~376,800 | ~3,140 |
| Stable memory writes | ~3,140 | ~3,140 |

**Estimated cycle reduction: 60-80%**

Conservative estimate: 2.1T → 400-800B per snapshot

## Migration Path

1. **Immediate:** Top up canister with cycles (currently at 2.6T, needs ~5T minimum buffer)
2. **Phase 1:** Implement inline metadata updates, remove `update_canister_summaries()`
3. **Phase 2:** Optimize burn calculation to be incremental
4. **Phase 3 (optional):** Optimize pruning with timestamp-first secondary index

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Canister runs out of cycles before fix | High | Top up immediately |
| Optimization introduces bugs | Medium | Test on local replica first |
| Incremental burns less accurate | Low | Can fall back to periodic full recalc |

## Additional Observations

- **Query success rate:** Only 1,785 of 3,140 canisters returned data (57% success). Many tracked canisters may be stopped, deleted, or have changed controllers. Consider pruning invalid canisters.
- **Idle burn:** 1.7B cycles/day idle cost is normal for stable memory usage.

## Conclusion

The current implementation has O(N × M) complexity for what should be O(N) operations. The fix is straightforward: update canister metadata inline during snapshot insertion rather than recalculating from scratch. This is a critical optimization - without it, the canister will consume ~50T cycles/day, which is unsustainable.
