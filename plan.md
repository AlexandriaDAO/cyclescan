# CycleScan - Cycles Burn Leaderboard

**CoinGecko for ICP cycle consumption.**

A simple leaderboard showing which canisters burn the most cycles, updated hourly.

---

## The Product

A single table:

| Canister | Project | Balance | 1h Burn | 24h Burn | 7d Burn |
|----------|---------|---------|---------|----------|---------|
| abc-cai | OpenChat | 50T | 1.2T | 28.8T | 201.6T |
| def-cai | DSCVR | 30T | 0.8T | 19.2T | 134.4T |
| ghi-cai | - | 10T | 0.1T | 2.4T | 16.8T |

That's it. Sorted by 24h burn by default.

---

## Architecture

### Data Flow

```
IC Dashboard API          CycleScan Canister              Frontend
       │                         │                            │
       │  fetch canister list    │                            │
       │  (offline, ~1M IDs)     │                            │
       │                         │                            │
       ▼                         │                            │
  trackable_canisters.json       │                            │
  (~1500 with blackhole          │                            │
   controllers)                  │                            │
       │                         │                            │
       │  import_canisters()     │                            │
       └────────────────────────►│                            │
                                 │                            │
                   ┌─────────────┴─────────────┐              │
                   │  Hourly: take_snapshot()  │              │
                   │  - Query each canister    │              │
                   │    via blackhole proxy    │              │
                   │  - Store cycle balance    │              │
                   │  - Prune >7 day old data  │              │
                   └─────────────┬─────────────┘              │
                                 │                            │
                                 │  get_leaderboard()         │
                                 │◄───────────────────────────┤
                                 │                            │
                                 │  [LeaderboardEntry, ...]   │
                                 ├───────────────────────────►│
                                 │                            │
```

### Backend (Rust Canister)

**Stable Memory Storage:**

1. **Canisters Map**
   - Key: `Principal` (canister_id)
   - Value: `CanisterMeta { proxy_id, project_name }`
   - Dynamic - can add/remove/update at runtime

2. **Snapshots Map**
   - Key: `(canister_id, timestamp)`
   - Value: `cycles: u128`
   - Rolling 7-day window, older entries pruned

**Public API:**

```candid
// Main query - the leaderboard
get_leaderboard : () -> (vec LeaderboardEntry) query;

// Trigger hourly snapshot
take_snapshot : () -> (SnapshotResult);

// Import canisters from JSON (admin)
import_canisters : (vec CanisterImport) -> (nat64);

// Set project name for a canister (admin)
set_project : (principal, opt text) -> ();

// Stats
get_stats : () -> (Stats) query;
```

**LeaderboardEntry:**
```
{
  canister_id: Principal,
  project: Option<String>,
  balance: u128,
  burn_1h: Option<u128>,
  burn_24h: Option<u128>,
  burn_7d: Option<u128>,
}
```

### Burn Calculation

For each time window (1h, 24h, 7d):
1. Find snapshots within that window
2. Take earliest and latest snapshot
3. If cycles decreased: `burn = earliest - latest`
4. If cycles increased (top-up): `burn = 0`

This is intentionally simple. Top-ups will cause under-counting, but hourly snapshots minimize the impact.

### Data Collection (Offline)

The canister list is collected offline and imported:

1. **Fetch all canisters** from IC Dashboard API
   ```bash
   ./data/fetch_batch.sh 0 25000
   ./data/fetch_batch.sh 25000 50000
   # ... etc
   ```

2. **Extract trackable canisters** (those with blackhole controllers)
   ```bash
   ./data/extract_trackable.sh canisters_0_25000.json
   # Combine results into trackable_canisters.json
   ```

3. **Import into canister**
   ```bash
   dfx canister call cyclescan_backend import_canisters "$(cat data/trackable_canisters.json | jq -c '[.[] | {canister_id: .canister_id, proxy_id: .proxy}]')"
   ```

### Blackhole Controllers

Canisters with these controllers expose their status publicly:

| Name | ID |
|------|-----|
| ninegua | `e3mmv-5qaaa-aaaah-aadma-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |
| CycleOps V1-V3 | Various (private to CycleOps) |
| Cygnus | `w7sux-siaaa-aaaai-qpasa-cai` |

We query `canister_status` through these proxies to get cycle balances.

---

## Frontend

Simple static site:
- Fetch leaderboard from backend
- Render as sortable table
- Click canister to see details/history (future)

---

## Deployment

Single command:
```bash
./scripts/deploy.sh
```

Deploys to mainnet. No local testing.

---

## Future Enhancements

1. **Project aggregation** - Sum burn across all canisters for a project
2. **Historical charts** - Burn rate over time
3. **Alerts** - Low balance warnings
4. **Automatic discovery** - Periodic re-scan for new trackable canisters
5. **Top-up tracking** - Separate top-ups from burn for accurate metrics
