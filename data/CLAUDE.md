# CycleScan Data Collection

## Files

```
data/
├── trackable_canisters.json   # Blackhole canisters (262)
├── sns_canisters.json         # SNS canisters (~350)
├── canisters_*_*.json         # Raw data from IC Dashboard API
├── fetch_batch.sh             # Fetch raw canister data
├── extract_trackable.sh       # Filter for public blackholes
└── fetch_sns.py               # Fetch all SNS canisters
```

## Data Sources

### 1. Blackhole Canisters

Canisters with ninegua or NNS Root as controller.

```bash
# Fetch raw data from IC Dashboard API
./fetch_batch.sh 0 25000

# Extract public blackhole canisters
./extract_trackable.sh canisters_0_25000.json

# Combine multiple ranges
jq -s 'add | unique_by(.canister_id)' public_*.json > trackable_canisters.json
```

**Output format:**
```json
[
  {"canister_id": "abc-cai", "proxy_id": "e3mmv-5qaaa-aaaah-aadma-cai"}
]
```

### 2. SNS Canisters

All canisters belonging to deployed SNSes.

```bash
# Fetch all SNS canisters
python3 fetch_sns.py
```

**Commands used internally:**
```bash
# List all deployed SNSes (52 as of Dec 2024)
dfx canister --network ic call qaa6y-5yaaa-aaaaa-aaafa-cai list_deployed_snses '(record {})'

# Get all canisters for a specific SNS
dfx canister --network ic call <sns_root_id> get_sns_canisters_summary '(record {})'
```

**Output format:**
```json
[
  {"canister_id": "abc-cai", "proxy_id": "zxeu2-7aaaa-aaaaq-aaafa-cai", "proxy_type": "SnsRoot"}
]
```

### 3. Combining for Import

```bash
# Add proxy_type to blackhole canisters and combine
jq '[.[] | . + {proxy_type: "Blackhole"}]' trackable_canisters.json > blackhole_import.json
jq -s 'add' blackhole_import.json sns_canisters.json > all_canisters.json
```

## Proxy Types

| Type | Query Method | Data Source |
|------|--------------|-------------|
| `Blackhole` | `canister_status(canister_id)` | IC Dashboard API + filter |
| `SnsRoot` | `get_sns_canisters_summary()` | SNS-W registry |

## Public Blackhole Controllers

| Name | ID | Notes |
|------|-----|-------|
| ninegua | `e3mmv-5qaaa-aaaah-aadma-cai` | Original blackhole |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` | System canister |

CycleOps V1/V2/V3 are private (reject non-CycleOps callers).

## Key Canister IDs

| Purpose | ID |
|---------|-----|
| SNS-W (SNS registry) | `qaa6y-5yaaa-aaaaa-aaafa-cai` |
| ninegua blackhole | `e3mmv-5qaaa-aaaah-aadma-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |

## Current Stats

| Source | Count |
|--------|-------|
| Blackhole (0-100k) | ~262 |
| SNS | ~350 |
| **Total** | **~612** |

## Project Research

Project mappings are stored in `project_mappings.json`. This file is **manually curated** and won't be overwritten by data scripts.

### Discovered Platforms (from ICRC-1 token names)

| Platform | Description | Count |
|----------|-------------|-------|
| ODIN.fun | Bioniq's Runes token launchpad (pump.fun for Bitcoin) | ~35 |
| FomoWell | Token launchpad | ~1 |
| Ordi Trade | Ordinals trading platform | ~2 |
| Unknown/Test | Generic or test tokens | ~10 |

### Research Methods

1. **ICRC-1 tokens**: Query `icrc1_name()` and `icrc1_symbol()`
2. **Pattern matching**: ODIN tokens end with "•ID•XXXX•ODIN"
3. **Web search**: Search canister IDs on GitHub, forums
4. **Dashboard lookup**: Check ICP Dashboard for metadata

### Non-token Canisters (~206)

Most blackhole canisters don't expose project names. They could be:
- NFT contracts
- DeFi protocols
- Static websites
- Backend services

These require manual research via:
- GitHub searches for canister ID
- Checking if canister hosts a frontend (https://CANISTER_ID.icp0.io)
- Querying candid interface for method names

## Adding New Data Sources

To add a new proxy type:

1. Create `fetch_<source>.py` script
2. Output JSON with `canister_id`, `proxy_id`, `proxy_type` fields
3. Add query logic to backend `lib.rs`
4. Document the query method here
