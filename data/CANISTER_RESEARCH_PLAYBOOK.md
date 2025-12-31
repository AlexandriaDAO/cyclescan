# Canister Research Playbook

**Purpose:** Systematically identify the project name for every unknown canister in CycleScan's database.

**Goal:** For each canister, determine either:
1. The **project name** (set via `set_project`)
2. A documented **reason** why identification failed (stored in `research_results_unknown.json`)

---

## Prerequisites

### Backend Canister
```
vohji-riaaa-aaaac-babxq-cai
```

### Identity
Use `daopad` identity for all commands.

### Output Files
- `research_results2.json` - Identified canisters (append new findings)
- `research_results_unknown.json` - Canisters that could not be identified (with reasons)

---

## Phase 1: Get the Full List of Unknown Canisters

### Step 1.1: Query the leaderboard for all entries with `project = null`

```bash
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai get_leaderboard '()'
```

Parse the output and extract all `canister_id` where `project = null`.

### Step 1.2: Load existing research files

Before researching, load:
- `research_results.json` - Already identified (skip these)
- `research_results2.json` - Already identified (skip these)
- `research_results_unknown.json` - Already documented as unknown (skip unless re-checking)

Create a working list of canisters that need research.

---

## Phase 2: Research Each Canister

For **each unknown canister**, execute these steps in order. Stop as soon as identification succeeds.

### Step 2.1: Query Candid Metadata

```bash
dfx canister --network ic metadata <CANISTER_ID> candid:service 2>&1 | head -100
```

**Analysis:**
- Look for distinctive type names, service names, or comments
- Common patterns to identify:

| Pattern in Candid | Project Type |
|-------------------|--------------|
| `icrc1_*` methods | ICRC-1 Token Ledger |
| `ledger_id` in InitArg | ICRC-1 Index Canister |
| `get_sns_canisters_summary` | SNS Root |
| `OrchestratorArg`, `AddErc20Arg` | ckERC20 Orchestrator |
| `EthereumNetwork`, `ckETH` | ckETH/ckERC20 Minter |
| `BitcoinNetwork`, `RuneBalance` | Bitcoin/Runes related |
| `Chain`, `Directive`, `cross-chain` | OmniBTC or bridge |
| `RequestPolicy`, `Station` | Orbit wallet |
| `spawn_miner`, `burned_cycles` | BoB Mining |
| `Proposal`, `Vote`, `Neuron` | DAO/Governance |
| `NFT`, `Token`, `Collection` | NFT project |
| `Swap`, `Pool`, `Liquidity` | DEX |

**If Candid unavailable:**
```
Error: Failed to read `candid:service` metadata
```
Document as: `"candid_available": false` and continue to next step.

### Step 2.2: Query ICRC-1 Token Methods

If this might be a token ledger:

```bash
dfx canister --network ic call <CANISTER_ID> icrc1_name '()' 2>&1
dfx canister --network ic call <CANISTER_ID> icrc1_symbol '()' 2>&1
```

**If successful:**
- Record token name and symbol
- Check if symbol matches known patterns:
  - `*•ID•*•ODIN` → Project: "ODIN.fun"
  - `ck*` → Project: "ck[Token] Ledger" (e.g., "ckETH Ledger")
  - Standard symbols → Search for project

### Step 2.3: Query Index Canister Methods

If Candid suggests this is an index:

```bash
dfx canister --network ic call <CANISTER_ID> ledger_id '()' 2>&1
```

**If successful:**
- Get the ledger principal
- Query the ledger for token name:
  ```bash
  dfx canister --network ic call <LEDGER_ID> icrc1_name '()' 2>&1
  dfx canister --network ic call <LEDGER_ID> icrc1_symbol '()' 2>&1
  ```
- Set project as "[Token] Index" (e.g., "ckUSDC Index")

### Step 2.4: Check ICP Dashboard

```bash
# Use WebFetch or browser
https://dashboard.internetcomputer.org/canister/<CANISTER_ID>
```

Extract:
- Canister name (if set)
- Controllers
- Subnet
- Module hash

**Controller analysis:**
| Controller | Indicates |
|------------|-----------|
| `r7inp-6aaaa-aaaaa-aaabq-cai` | NNS Root - system/infrastructure canister |
| `e3mmv-5qaaa-aaaah-aadma-cai` | ninegua blackhole - community canister |
| `vxkom-oyaaa-aaaar-qafda-cai` | ckERC20 Orchestrator - ckERC20 component |
| SNS root pattern | SNS DAO canister |

### Step 2.5: Web Search

```bash
# Search patterns to try:
"<CANISTER_ID>" site:forum.dfinity.org
"<CANISTER_ID>" site:github.com
"<CANISTER_ID>" ICP canister
"<CANISTER_ID>" Internet Computer
```

Look for:
- Forum announcements
- GitHub repositories
- Project documentation
- News articles

### Step 2.6: Query Common Public Methods

Try these common query methods:

```bash
# Health/status
dfx canister --network ic call <CANISTER_ID> health_status '()' 2>&1
dfx canister --network ic call <CANISTER_ID> get_status '()' 2>&1

# Info methods
dfx canister --network ic call <CANISTER_ID> get_info '()' 2>&1
dfx canister --network ic call <CANISTER_ID> info '()' 2>&1
dfx canister --network ic call <CANISTER_ID> get_metadata '()' 2>&1

# Stats
dfx canister --network ic call <CANISTER_ID> get_stats '()' 2>&1
dfx canister --network ic call <CANISTER_ID> stats '()' 2>&1
dfx canister --network ic call <CANISTER_ID> get_statistics '()' 2>&1

# Config
dfx canister --network ic call <CANISTER_ID> get_config '()' 2>&1
dfx canister --network ic call <CANISTER_ID> config '()' 2>&1

# HTTP interface (may return useful info)
dfx canister --network ic call <CANISTER_ID> http_request '(record { url = "/"; method = "GET"; body = blob ""; headers = vec {}; certificate_version = opt 1 })' 2>&1
```

### Step 2.7: Check Related Canisters

If controllers include other canisters:
1. Research those controller canisters
2. They may reveal the project (e.g., an Orbit Upgrader reveals an Orbit Station)

If the canister controls other canisters:
1. Those might be more identifiable
2. Use the relationship to infer the project

---

## Phase 3: Record Findings

### For Identified Canisters

**Immediately set the project in the backend:**
```bash
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_project '(principal "<CANISTER_ID>", opt "<PROJECT_NAME>")'
```

**Append to `research_results2.json`:**
```json
{
  "canister_id": "<CANISTER_ID>",
  "project": "<PROJECT_NAME>",
  "notes": "<How identified, any extra context>",
  "category": "<infrastructure|nns_infrastructure|defi|token|nft|social|gaming|unknown>",
  "is_token": true/false,
  "token_name": "<if applicable>",
  "token_symbol": "<if applicable>",
  "burn_rank": <position in leaderboard>
}
```

### For Unidentified Canisters

**Append to `research_results_unknown.json`:**
```json
{
  "canister_id": "<CANISTER_ID>",
  "project": null,
  "reason": "<specific reason>",
  "candid_available": true/false,
  "candid_summary": "<brief description of interface if available>",
  "controllers": ["<controller1>", "<controller2>"],
  "status": "<enabled|disabled>",
  "methods_tried": ["icrc1_name", "ledger_id", "health_status"],
  "dashboard_info": "<any info from dashboard>",
  "web_search_result": "<summary of search results or 'no results'>",
  "burn_rank": <position in leaderboard>,
  "last_researched": "<ISO date>"
}
```

**Reason codes:**
- `no_candid_metadata` - No Candid interface exposed
- `candid_unrecognizable` - Has Candid but pattern unknown
- `all_methods_unauthorized` - Methods exist but require auth
- `canister_disabled` - Canister is not running
- `no_public_info` - No info found via any method
- `generic_infrastructure` - Appears to be generic infra (archive, etc.)

---

## Phase 4: Known Project Patterns

### NNS/System Infrastructure
Set project based on function:
- ckERC20 Orchestrator
- ckETH Minter / ckBTC Minter
- ck[TOKEN] Ledger (e.g., ckUSDC Ledger)
- ck[TOKEN] Index
- Cycles Minting Canister
- NNS Governance
- ICP Ledger

### SNS DAOs
Query SNS root to get canister summary, then label all canisters with the SNS project name.

Already mapped in `sns_canisters.json`.

### OmniBTC
Multiple canisters with cross-chain functionality:
- Hub, Customs, Routes (ICP, EVM, Doge, Solana)
- Runes Indexer, Runes Exchange
All should be labeled "OmniBTC"

### Token Launchpads
- **ODIN.fun** - Tokens with `•ID•*•ODIN` pattern
- **FomoWell** - Check token registry
- **Ordi Trade** - Check token registry

### Known Projects to Watch For
| Project | Identifying Features |
|---------|---------------------|
| Sonic | DEX, swap methods |
| ICPSwap | DEX, liquidity pools |
| KongSwap | DEX |
| OpenChat | CHAT token, messaging |
| Nuance | Blogging platform |
| NFID | Wallet, identity |
| Yuku | NFT marketplace |
| Entrepot | NFT marketplace |
| ORIGYN | NFT certification |
| Kinic | Search engine |
| Hot or Not | Social |
| DSCVR | Social |
| Distrikt | Social |
| Catalyze | DAO tooling |
| ICLighthouse | DeFi tools |
| WaterNeuron | Liquid staking |
| GoldDAO | Gold-backed token |
| Neutrinite | NTN token ecosystem |
| Draggin Karma | DKP rewards |

---

## Phase 5: Batch Processing Strategy

### Priority Order
Research canisters in order of cycle burn (highest first):
1. Top 100 by 24h burn
2. Top 100 by 7d burn
3. Remaining by balance

### Parallelization
For efficiency, batch similar operations:
1. First pass: Query all Candid metadata
2. Second pass: Query all ICRC-1 methods
3. Third pass: Query all index ledger_id methods
4. Fourth pass: Web searches for remaining unknowns

### Progress Tracking
Maintain a progress file:
```json
{
  "total_canisters": 3138,
  "researched": 2500,
  "identified": 2100,
  "unknown_documented": 400,
  "remaining": 638,
  "last_canister_id": "xxx-cai",
  "last_updated": "2025-12-31T12:00:00Z"
}
```

---

## Phase 6: Output Validation

### After completing all research:

1. **Verify no duplicates** in research files
2. **Verify all identified** canisters have `set_project` called
3. **Verify coverage:**
   ```bash
   # Count unknowns remaining in leaderboard
   dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai get_leaderboard '()' | grep "project = null" | wc -l
   ```
4. **Generate summary stats:**
   - Total canisters
   - Identified by category
   - Unknown by reason code

---

## Quick Reference Commands

```bash
# Get Candid
dfx canister --network ic metadata <CID> candid:service 2>&1 | head -100

# ICRC-1 token info
dfx canister --network ic call <CID> icrc1_name '()'
dfx canister --network ic call <CID> icrc1_symbol '()'

# Index → Ledger lookup
dfx canister --network ic call <CID> ledger_id '()'

# Set project (after identification)
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_project '(principal "<CID>", opt "<PROJECT>")'

# Verify project set
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai get_leaderboard '()' | grep "<CID>"
```

---

## Example Workflow

```
1. Get unknown canister: abc-cai
2. Query Candid → Shows ICRC-1 ledger interface
3. Call icrc1_name → Returns "ckSHIB"
4. Identify as ckERC20 token
5. Set project:
   dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_project '(principal "abc-cai", opt "ckSHIB Ledger")'
6. Record in research_results2.json
7. Move to next canister
```

---

## Success Criteria

Research is complete when:
- [ ] Every canister in the leaderboard has either a project name OR is documented in `research_results_unknown.json`
- [ ] All identified canisters have `set_project` called on the backend
- [ ] Unknown canisters have specific, documented reasons for being unknown
- [ ] No canister is left with `project = null` without documentation

---

## Appendix: Known Canister Prefixes by Subnet

Canisters on the same subnet often belong to the same project. Track patterns:

| Subnet | Common Projects |
|--------|-----------------|
| `pzp6e-*` | Various |
| `fuqsr-*` | ckERC20 infrastructure |
| `w4rem-*` | SNS DAOs |

Use subnet clustering as a hint when individual research fails.
