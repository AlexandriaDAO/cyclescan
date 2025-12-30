# Cycles Burn Tracker - Data Collection

This folder contains canister data for tracking cycle consumption across ICP.

## Project Goal

Track cycle burn rates for all ICP canisters that have opted into public monitoring via blackhole controllers. This involves:

1. **Collect** - Fetch all ~1M canister IDs with their controllers
2. **Filter** - Identify canisters with blackhole controllers (~1-2% of total)
3. **Snapshot** - Periodically capture cycle balances for trackable canisters
4. **Analyze** - Calculate burn rates and display in dashboard

## Data Source

**IC Dashboard API v3**
- Base: `https://ic-api.internetcomputer.org/api/v3/`
- Endpoint: `/canisters?limit=100&offset=N`
- Max limit: 100 per request
- Total canisters: ~986,000 (Dec 2024)

Each API response includes full canister metadata:
```json
{
  "canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
  "controllers": ["e3mmv-5qaaa-aaaah-aadma-cai", "..."],
  "subnet_id": "...",
  "module_hash": "...",
  "language": "rust",
  "name": "",
  "enabled": true,
  "updated_at": "2025-12-29T..."
}
```

## Blackhole Controllers

Canisters with these controllers can be publicly queried for status:

| Name | Canister ID |
|------|-------------|
| ninegua Original | `e3mmv-5qaaa-aaaah-aadma-cai` |
| CycleOps V1 | `5vdms-kaaaa-aaaap-aa3uq-cai` |
| CycleOps V2 | `2daxo-giaaa-aaaap-anvca-cai` |
| CycleOps V3 | `cpbhu-5iaaa-aaaad-aalta-cai` |
| Cygnus | `w7sux-siaaa-aaaai-qpasa-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |

## Data Collection Strategy

### Step 1: Fetch All Canisters (Current)

Fetch canister metadata from IC Dashboard API in batches:

```bash
# Fetch batch (100 canisters)
curl -s "https://ic-api.internetcomputer.org/api/v3/canisters?limit=100&offset=0"
```

Save as JSON files:
- `canisters_0_25000.json` - First 25k canisters
- `canisters_25000_50000.json` - Next 25k
- etc.

**Performance**: ~1M canisters / 100 per batch = 10,000 API calls (~1 hour)

### Step 2: Filter for Trackable Canisters

Filter locally using jq:

```bash
BLACKHOLES="e3mmv-5qaaa-aaaah-aadma-cai|5vdms-kaaaa-aaaap-aa3uq-cai|..."
jq -c '.data[] | select(any(.controllers[]?; test("'$BLACKHOLES'")))' canisters_*.json
```

**Expected yield**: ~1-2% = 10,000-20,000 trackable canisters

### Step 3: Query Cycle Balances (Future)

For trackable canisters, query status via blackhole proxy:

```bash
dfx canister --network ic call <proxy_id> canister_status \
  '(record { canister_id = principal "<canister_id>" })'
```

Returns: cycles, memory_size, status, module_hash

### Step 4: Store Snapshots (Future)

SQLite database schema:

```sql
CREATE TABLE canisters (
    canister_id TEXT PRIMARY KEY,
    proxy_id TEXT NOT NULL,
    controllers TEXT,
    first_seen DATETIME,
    last_updated DATETIME
);

CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY,
    canister_id TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    cycles BIGINT NOT NULL,
    memory_size BIGINT,
    status TEXT
);
```

## File Structure

```
data/
├── CLAUDE.md                      # This file
├── canisters_0_25000.json         # Raw canister data (batches)
├── canisters_25000_50000.json
├── ...
├── trackable_canisters.json       # Filtered: only blackhole-controlled
└── cycles_tracker.db              # SQLite for snapshots (future)
```

## Progress

| Range | Status | Trackable Found |
|-------|--------|-----------------|
| 0 - 25,000 | Pending | - |
| 25,000 - 50,000 | Pending | - |
| ... | ... | ... |

## Commands Reference

**Fetch a batch:**
```bash
curl -s "https://ic-api.internetcomputer.org/api/v3/canisters?limit=100&offset=N" >> batch.json
```

**Count trackable in a file:**
```bash
BLACKHOLES="e3mmv-5qaaa-aaaah-aadma-cai|5vdms-kaaaa-aaaap-aa3uq-cai|2daxo-giaaa-aaaap-anvca-cai|cpbhu-5iaaa-aaaad-aalta-cai|w7sux-siaaa-aaaai-qpasa-cai|r7inp-6aaaa-aaaaa-aaabq-cai"
jq "[.data[] | select(any(.controllers[]?; test(\"$BLACKHOLES\")))] | length" file.json
```

**Query canister status via blackhole:**
```bash
dfx canister --network ic call e3mmv-5qaaa-aaaah-aadma-cai canister_status \
  '(record { canister_id = principal "ryjl3-tyaaa-aaaaa-aaaba-cai" })'
```
