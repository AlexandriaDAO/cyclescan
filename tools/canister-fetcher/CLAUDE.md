# Data Fetching - Agent Notes

This directory handles all data collection for the Cycles Burn Tracker (see `/plan.md`).

## Long-Term Data Structure

```
tools/canister-fetcher/
├── CLAUDE.md                     # This file
├── Cargo.toml                    # Rust CLI tool config
├── src/
│   └── main.rs                   # Canister ID fetcher
├── data/
│   ├── 01_all_canisters/         # Step 1: Raw canister ID lists
│   │   ├── canisters_0_25000.txt
│   │   ├── canisters_25000_50000.txt
│   │   ├── canisters_50000_75000.txt
│   │   └── ...                   # ~40 files for ~985k canisters
│   │
│   ├── 02_trackable/             # Step 2: Canisters with blackhole controllers
│   │   ├── trackable_canisters.json
│   │   └── scan_progress.json    # Resume state for blackhole scanning
│   │
│   └── 03_database/              # Steps 3-4: SQLite for ongoing tracking
│       └── cycles_tracker.db
│
└── fetcher_state.json            # Current fetch state (temp, per-session)
```

## Step 1: Fetch All Canister IDs

**Status**: In progress

Uses IC Dashboard API v3 with offset-based pagination:
```
https://ic-api.internetcomputer.org/api/v3/canisters?limit=100&offset=N
```

### Current Progress

| File | Offset Range | Count | Status |
|------|--------------|-------|--------|
| `data/01_all_canisters/canisters_0_25000.txt` | 0 - 24,999 | 25,000 | Done |
| `data/01_all_canisters/canisters_25000_50000.txt` | 25,000 - 49,999 | - | Pending |
| ... | ... | ... | ... |

Total on network: ~985,515 canisters (Dec 2024)

### Running the Fetcher

```bash
# Build (from project root)
cargo build --release -p canister-fetcher

# Run from this directory
cd tools/canister-fetcher
../../target/release/canister-fetcher --count 25000
```

### Saving a Chunk

After fetching completes (from `tools/canister-fetcher/`):
```bash
# Dedupe and save (replace X and Y with actual offsets)
cat fetcher_state.json | jq -r '.canisters[]' | awk '!seen[$0]++' \
  > data/01_all_canisters/canisters_25000_50000.txt

# Reset state for next chunk
echo '{"offset": 50000, "canisters": []}' > fetcher_state.json
```

### Known Issues

- **Duplicates on resume**: If fetcher times out mid-batch and resumes, ~10-15 duplicate IDs may appear at the boundary. Always dedupe when saving.
- **v4 API broken**: The cursor parameter is ignored. Use v3 with offset/limit.

## Step 2: Identify Trackable Canisters (Future)

Will use `dfx` to query each canister through blackhole proxies:

```bash
dfx canister --network ic call <proxy_id> canister_status \
  '(record { canister_id = principal "<canister_id>" })'
```

### Blackhole Proxies

| Name | Canister ID |
|------|-------------|
| ninegua Original | `e3mmv-5qaaa-aaaah-aadma-cai` |
| CycleOps V1 | `5vdms-kaaaa-aaaap-aa3uq-cai` |
| CycleOps V2 | `2daxo-giaaa-aaaap-anvca-cai` |
| CycleOps V3 | `cpbhu-5iaaa-aaaad-aalta-cai` |
| Cygnus | `w7sux-siaaa-aaaai-qpasa-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |

If call succeeds, the canister is trackable via that proxy.

### Expected Results

- ~985k canisters scanned
- ~1-5% have blackhole controllers
- **~10,000 - 50,000 trackable canisters**

## Step 3-4: Database & Snapshots (Future)

SQLite database at `data/03_database/cycles_tracker.db`:

- `canisters` table: trackable canister metadata
- `snapshots` table: periodic cycle balance readings

See `/plan.md` for schema details.

## API Reference

### IC Dashboard API v3

- Base: `https://ic-api.internetcomputer.org/api/v3/`
- Canisters: `/canisters?limit=100&offset=N`
- Max limit: 100
- No rate limiting observed

### Canister Status (via blackhole)

```
dfx canister --network ic call <proxy> canister_status '(record { canister_id = principal "<id>" })'
```

Returns: cycles, memory_size, status, controllers, module_hash
