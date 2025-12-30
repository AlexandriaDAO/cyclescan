# CycleScan - Cycles Burn Leaderboard

**CoinGecko for ICP cycle consumption.**

A leaderboard showing which canisters burn the most cycles, updated hourly.

---

## The Product

| Canister | Project | Balance | 1h Burn | 24h Burn | 7d Burn |
|----------|---------|---------|---------|----------|---------|
| abc-cai | OpenChat | 50T | 1.2T | 28.8T | 201.6T |
| def-cai | DSCVR | 30T | 0.8T | 19.2T | 134.4T |

Sorted by 24h burn. That's it.

---

## Architecture

### Extensible Proxy System

Different canisters expose their cycles in different ways. We handle this with a `ProxyType` enum:

```rust
enum ProxyType {
    Blackhole,  // Query via canister_status(canister_id)
    SnsRoot,    // Query via get_sns_canisters_summary()
    // Future: add new variants for other query methods
}
```

Each canister is stored with:
- `canister_id` - The canister to track
- `proxy_id` - The canister to query for cycles
- `proxy_type` - How to query it
- `project_name` - Optional project association

### Data Sources

| Source | Proxy Type | Query Method | Count |
|--------|------------|--------------|-------|
| Blackhole (ninegua) | `Blackhole` | `canister_status(canister_id)` | ~116 |
| Blackhole (NNS Root) | `Blackhole` | `canister_status(canister_id)` | ~146 |
| SNS Projects | `SnsRoot` | `get_sns_canisters_summary()` | ~350 |
| **Total** | | | **~612** |

### Snapshot Flow

```
take_snapshot()
      │
      ├─► Group canisters by proxy_type
      │
      ├─► Blackhole canisters:
      │      For each canister:
      │        call proxy.canister_status(canister_id)
      │        store cycles
      │
      └─► SNS canisters:
             For each unique SNS root:
               call root.get_sns_canisters_summary()
               extract cycles for all canisters in that SNS
               store cycles
```

---

## Data Collection

### 1. Blackhole Canisters (Complete)

Canisters with public blackhole controllers (ninegua or NNS Root).

**Source:** IC Dashboard API
**Script:** `data/extract_trackable.sh`
**Output:** `data/trackable_canisters.json` (262 canisters)

```bash
# Already done for 0-100k range
cd data
./fetch_batch.sh 0 25000
./extract_trackable.sh canisters_0_25000.json
```

### 2. SNS Canisters (TODO)

All canisters belonging to deployed SNS projects.

**Source:** SNS-W registry (`qaa6y-5yaaa-aaaaa-aaafa-cai`)
**Script:** `data/fetch_sns.py` (created but slow)

**The commands:**
```bash
# List all 52 deployed SNSes
dfx canister --network ic call qaa6y-5yaaa-aaaaa-aaafa-cai list_deployed_snses '(record {})'

# For each SNS root, get all its canisters with cycles
dfx canister --network ic call <sns_root_id> get_sns_canisters_summary '(record {})'
```

**Response structure:**
```
{
  root: { canister_id, status: { cycles } },
  governance: { canister_id, status: { cycles } },
  ledger: { canister_id, status: { cycles } },
  index: { canister_id, status: { cycles } },
  swap: { canister_id, status: { cycles } },
  dapps: [ { canister_id, status: { cycles } }, ... ],
  archives: [ { canister_id, status: { cycles } }, ... ]
}
```

**Manual approach (if script times out):**

1. Get list of SNS roots:
```bash
dfx canister --network ic call qaa6y-5yaaa-aaaaa-aaafa-cai list_deployed_snses '(record {})' > sns_list.txt
grep -oE 'root_canister_id = opt principal "[a-z0-9-]+-cai"' sns_list.txt | sed 's/.*"//;s/"//' > sns_roots.txt
```

2. For each root, manually query and extract canister IDs:
```bash
dfx canister --network ic call zxeu2-7aaaa-aaaaq-aaafa-cai get_sns_canisters_summary '(record {})'
```

3. Build JSON manually or in batches.

---

## Import Format

The backend expects this format for `import_canisters()`:

```json
[
  {
    "canister_id": "abc-xyz-cai",
    "proxy_id": "e3mmv-5qaaa-aaaah-aadma-cai",
    "proxy_type": "Blackhole"
  },
  {
    "canister_id": "def-uvw-cai",
    "proxy_id": "zxeu2-7aaaa-aaaaq-aaafa-cai",
    "proxy_type": "SnsRoot"
  }
]
```

**Candid format:**
```
(vec {
  record {
    canister_id = principal "abc-xyz-cai";
    proxy_id = principal "e3mmv-...";
    proxy_type = variant { Blackhole }
  };
  record {
    canister_id = principal "def-uvw-cai";
    proxy_id = principal "zxeu2-...";
    proxy_type = variant { SnsRoot }
  };
})
```

---

## TODO

### Immediate
- [ ] Fetch SNS canister data (manually or fix script timeout)
- [ ] Combine blackhole + SNS data into single import file
- [ ] Deploy updated backend
- [ ] Import all canisters
- [ ] Take first snapshot
- [ ] Verify leaderboard works

### Future
- [ ] Set up hourly cron for `take_snapshot()`
- [ ] Build simple frontend
- [ ] Add project name mappings
- [ ] Expand blackhole scan beyond 100k range
- [ ] Add more proxy types as discovered

---

## Adding New Proxy Types

When a new canister type exposes cycles differently:

1. **Add variant to ProxyType** (`lib.rs:27`):
```rust
enum ProxyType {
    Blackhole,
    SnsRoot,
    NewType,  // Add here
}
```

2. **Add query function** (`lib.rs`):
```rust
async fn query_new_type(canister_id: Principal, proxy_id: Principal) -> CallResult<u128> {
    // Implement query logic
}
```

3. **Update take_snapshot()** to handle new type

4. **Update Storable byte mapping** (`lib.rs:106`):
```rust
let proxy_type_byte: u8 = match self.proxy_type {
    ProxyType::Blackhole => 0,
    ProxyType::SnsRoot => 1,
    ProxyType::NewType => 2,  // Add here
};
```

5. **Create data fetching script** in `data/`

6. **Update .did file** with new variant

---

## Key Canister IDs

| Purpose | ID |
|---------|-----|
| CycleScan Backend | `vohji-riaaa-aaaac-babxq-cai` |
| ninegua blackhole | `e3mmv-5qaaa-aaaah-aadma-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |
| SNS-W (registry) | `qaa6y-5yaaa-aaaaa-aaafa-cai` |

---

## Why Only These Sources?

**Blackhole canisters:** Only ninegua and NNS Root are publicly queryable. CycleOps V1/V2/V3 blackholes have access control that rejects non-CycleOps callers.

**SNS canisters:** The SNS root canister can query status of all canisters in its SNS because it's their controller. We call `get_sns_canisters_summary()` which returns cycles for root, governance, ledger, index, swap, dapps, and archives.

**Other canisters:** ~99% of canisters have no public way to query their cycles. We can only track canisters that have opted into transparency via blackhole controllers or SNS governance.
