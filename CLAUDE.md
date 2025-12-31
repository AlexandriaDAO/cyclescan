# CycleScan

Cycles burn leaderboard for ICP. Like CoinGecko, but for cycle consumption.

## Canister IDs (Mainnet)

| Canister | ID |
|----------|-----|
| Backend | `vohji-riaaa-aaaac-babxq-cai` |

- Dashboard: https://dashboard.internetcomputer.org/canister/vohji-riaaa-aaaac-babxq-cai
- Candid UI: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=vohji-riaaa-aaaac-babxq-cai

## Development

**Always deploy after changes** (no local dev environment):
```bash
./scripts/deploy.sh
```

Uses `daopad` identity.

## Architecture

**Backend stores canisters with their proxy type and method:**

```rust
enum ProxyType {
    Blackhole,  // Query via canister_status(canister_id)
    SnsRoot,    // Query via get_sns_canisters_summary()
    // Future: add new variants for other query methods
}
```

**Stable memory:**
1. **Canisters** - `canister_id -> { proxy_id, proxy_type, project_name }`
2. **Snapshots** - `(canister_id, timestamp) -> cycles`

**Hourly workflow:**
1. Call `take_snapshot()`
2. Backend groups canisters by proxy_type
3. Blackhole: query `canister_status` per canister
4. SnsRoot: query `get_sns_canisters_summary` per SNS (returns all canisters)
5. Store cycles, prune >7 day old data

## API

```candid
get_leaderboard : () -> (vec LeaderboardEntry) query;
take_snapshot : () -> (SnapshotResult);
import_canisters : (vec CanisterImport) -> (nat64);
set_project : (principal, opt text) -> ();
get_stats : () -> (Stats) query;
```

## Data Sources

### 1. Blackhole Canisters (ninegua, NNS Root)

```bash
# Fetch from IC Dashboard API
cd data
./fetch_batch.sh 0 25000
./extract_trackable.sh canisters_0_25000.json
```

### 2. SNS Canisters

```bash
# List all deployed SNSes
dfx canister --network ic call qaa6y-5yaaa-aaaaa-aaafa-cai list_deployed_snses '(record {})'

# Get all canisters for a specific SNS
dfx canister --network ic call <sns_root_id> get_sns_canisters_summary '(record {})'

# Fetch all SNS canisters
cd data
python3 fetch_sns.py
```

### 3. Adding New Proxy Types (Future)

To add support for a new canister query method:

1. **Add variant to ProxyType enum** in `lib.rs`:
   ```rust
   enum ProxyType {
       Blackhole,
       SnsRoot,
       NewMethod,  // Add here
   }
   ```

2. **Add query function**:
   ```rust
   async fn query_new_method(canister_id: Principal) -> CallResult<u128> {
       // Implement the query
   }
   ```

3. **Update take_snapshot()** to handle the new type

4. **Update Storable impl** for CanisterMeta (add byte mapping)

5. **Create data fetching script** in `data/`

## Import Format

```json
[
  {"canister_id": "abc-cai", "proxy_id": "e3mmv-...", "proxy_type": "Blackhole"},
  {"canister_id": "def-cai", "proxy_id": "zxeu2-...", "proxy_type": "SnsRoot"}
]
```

## Proxy Types

| Type | Proxy | Query Method | Notes |
|------|-------|--------------|-------|
| Blackhole | ninegua, NNS Root | `canister_status(canister_id)` | One call per canister |
| SnsRoot | SNS root canister | `get_sns_canisters_summary()` | One call returns all SNS canisters |

## Key Canister IDs

| Purpose | ID |
|---------|-----|
| ninegua blackhole | `e3mmv-5qaaa-aaaah-aadma-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |
| SNS-W (SNS registry) | `qaa6y-5yaaa-aaaaa-aaafa-cai` |
