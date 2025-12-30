# CycleScan

Cycles burn leaderboard for ICP. Like CoinGecko, but for cycle consumption.

## Canister IDs (Mainnet)

| Canister | ID |
|----------|-----|
| Backend | `vohji-riaaa-aaaac-babxq-cai` |

- Dashboard: https://dashboard.internetcomputer.org/canister/vohji-riaaa-aaaac-babxq-cai
- Candid UI: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=vohji-riaaa-aaaac-babxq-cai

## Development

**Deploy to mainnet:**
```bash
./scripts/deploy.sh
```

No local testing. Uses `daopad` identity.

## Architecture

**Backend stores two things in stable memory:**

1. **Canisters** - `canister_id -> { proxy_id, project_name }`
2. **Snapshots** - `(canister_id, timestamp) -> cycles`

**Hourly workflow:**
1. Someone calls `take_snapshot()`
2. Backend queries each canister via its blackhole proxy
3. Stores cycle balance with timestamp
4. Old snapshots (>7 days) get pruned

## API

```candid
// The leaderboard - main query
get_leaderboard : () -> (vec LeaderboardEntry) query;

// Trigger snapshot (anyone can call)
take_snapshot : () -> (SnapshotResult);

// Import canisters (controller only)
import_canisters : (vec CanisterImport) -> (nat64);

// Set project name (controller only)
set_project : (principal, opt text) -> ();

// Stats
get_stats : () -> (Stats) query;
```

## Data Files

```
data/
├── trackable_canisters.json      # Main list to import (~1500 canisters)
├── canisters_*_*.json            # Raw data from IC Dashboard API
├── trackable_canisters_*_*.json  # Per-range filtered data
├── fetch_batch.sh                # Fetch canisters from API
└── extract_trackable.sh          # Filter for blackhole controllers
```

## Updating Canister List

1. Fetch new data:
   ```bash
   cd data
   ./fetch_batch.sh 200000 225000
   ./extract_trackable.sh canisters_200000_225000.json
   ```

2. Merge into main list:
   ```bash
   jq -s 'add | unique_by(.canister_id)' trackable_*.json > trackable_canisters.json
   ```

3. Import to canister:
   ```bash
   # Format for import
   cat trackable_canisters.json | jq -c '[.[] | {c: .canister_id, p: .proxy}]' > import.json

   # Call import (will need to batch if large)
   dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai import_canisters "(vec { ... })"
   ```

## Blackhole Controllers

| Name | ID | Works? |
|------|-----|--------|
| ninegua | `e3mmv-5qaaa-aaaah-aadma-cai` | Yes |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` | Yes |
| CycleOps V1-V3 | Various | No (private) |
| Cygnus | `w7sux-siaaa-aaaai-qpasa-cai` | Unknown |
