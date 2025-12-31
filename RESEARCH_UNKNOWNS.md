# Manual Unknown Canister Research

**Objective:** Manually research every canister with `project = null` and either identify the project or mark it as `"Unidentified"` so we don't re-research it.

---

## How This Works

1. **Pick a range** (e.g., ranks 1-100, 100-200, etc.)
2. **Query the leaderboard** and filter for `project = null` in that range
3. **For each unknown**, manually investigate using the methods below
4. **Set the project** to either the identified name OR `"Unidentified"` if truly unknown
5. **Move to the next** until the range is complete

---

## Step 1: Get Unknowns in Your Range

Replace `RANGE_START` and `RANGE_END` with your assigned range.

```bash
# Get leaderboard and count position
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai get_leaderboard '()' 2>&1 > /tmp/leaderboard.txt

# Manually scan for `project = null` entries in positions RANGE_START to RANGE_END
# Or parse the file to extract just those entries
```

The leaderboard is ordered by cycle burn (highest first), so:
- Ranks 1-100 = Top 100 cycle burners
- Ranks 100-200 = Next 100
- etc.

---

## Step 2: Research Each Unknown Canister

For each canister with `project = null`, perform these steps **in order**. Stop when you identify the project.

### 2.1 Get the Candid Interface

```bash
dfx canister --network ic metadata <CANISTER_ID> candid:service 2>&1
```

**Read the output carefully.** Look for:
- Type names that hint at functionality (e.g., `SwapArgs`, `NFTMetadata`, `TokenInfo`)
- Method names (e.g., `icrc1_transfer`, `get_neurons`, `swap`)
- Comments in the interface
- Service name

**Common patterns:**

| You see... | It's probably... |
|------------|------------------|
| `icrc1_*` methods, `Tokens`, `Account` | Token ledger - call `icrc1_name` and `icrc1_symbol` |
| `ledger_id` in InitArg | Index canister - call `ledger_id()` to find parent token |
| `get_sns_*`, `Neuron`, `Proposal` | SNS DAO component |
| `Chain`, `Directive`, `cross-chain` | Bridge (likely OmniBTC or Omnity) |
| `Station`, `RequestPolicy`, `Quorum` | Orbit wallet |
| `spawn_miner`, `burn`, `pool` | Mining/DeFi mechanism |
| `Swap`, `Pool`, `Liquidity` | DEX |
| `NFT`, `Collection`, `TokenMetadata` | NFT project |

### 2.2 Call Common Query Methods

Based on what the Candid shows, call relevant methods.

**⚠️ IMPORTANT: dfx WARNING PARSING BUG**

When a canister doesn't expose `candid:service` metadata, `dfx` outputs warnings to **stdout** (not stderr!):

```
WARN: Cannot fetch Candid interface for icrc1_name, sending arguments with inferred types.
("Token")
```

**Only use the LAST LINE** (the value in parentheses). Never copy-paste the full output as a project name!

**Correct:** `Token`
**Wrong:** `WARN: Cannot fetch Candid interface for icrc1_name, sending arguments with inferred types. Token`

```bash
# For tokens:
dfx canister --network ic call <CANISTER_ID> icrc1_name '()'
dfx canister --network ic call <CANISTER_ID> icrc1_symbol '()'

# For index canisters:
dfx canister --network ic call <CANISTER_ID> ledger_id '()'
# Then look up that ledger

# For general info:
dfx canister --network ic call <CANISTER_ID> get_info '()'
dfx canister --network ic call <CANISTER_ID> get_config '()'
dfx canister --network ic call <CANISTER_ID> get_stats '()'
dfx canister --network ic call <CANISTER_ID> health_status '()'

# HTTP endpoint (sometimes has useful info):
dfx canister --network ic call <CANISTER_ID> http_request '(record { url = "/"; method = "GET"; body = blob ""; headers = vec {}; certificate_version = opt 1 })'
```

### 2.3 Check the Dashboard

Visit: `https://dashboard.internetcomputer.org/canister/<CANISTER_ID>`

Note:
- **Controllers** - Who controls it? Other canisters? Principals?
- **Subnet** - Canisters on same subnet often belong to same project
- **Module hash** - Same hash = same code = related canisters

### 2.4 Web Search

Search for the canister ID:
```
"<CANISTER_ID>" site:forum.dfinity.org
"<CANISTER_ID>" site:github.com
"<CANISTER_ID>" Internet Computer
```

Also search for any distinctive method names or types you found in the Candid.

### 2.5 Check Controller Canisters

If the canister is controlled by another canister, research that controller:
```bash
dfx canister --network ic metadata <CONTROLLER_CANISTER_ID> candid:service 2>&1
```

The controller often reveals the project (e.g., an SNS root, an Orbit upgrader).

---

## Step 3: Set the Project

### If you identified the project:
```bash
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_project '(principal "<CANISTER_ID>", opt "<PROJECT_NAME>")'
```

**Naming conventions:**
- For tokens: `"<TOKEN_SYMBOL> Ledger"` (e.g., `"ckUSDC Ledger"`)
- For indexes: `"<TOKEN_SYMBOL> Index"` (e.g., `"ckUSDC Index"`)
- For SNS components: Use the SNS project name
- For multi-canister projects: Use the main project name (e.g., `"OmniBTC"`, `"Orbit"`)

### If you could NOT identify the project:

Use one of the standard categories below. This marks it as **vetted** so we don't waste time re-researching.

```bash
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_project '(principal "<CANISTER_ID>", opt "<CATEGORY>")'
```

---

## Standard Categories for Unidentified Canisters

Use the most specific category that fits. If you need a new category, add it here for future researchers.

| Category | When to Use |
|----------|-------------|
| `Unidentified` | Generic - tried everything, can't identify |
| `Unidentified (no candid)` | No `candid:service` metadata exposed |
| `Unidentified (disabled)` | Canister is stopped or frozen |
| `Unidentified (auth required)` | Methods exist but require authentication |
| `Abandoned` | Canister is out of cycles (dead) |
| `Unknown Token` | ICRC-1 token but can't identify the project |
| `Asset Canister` | Frontend asset canister (serves static files) |
| `Logger Canister` | Logging/monitoring infrastructure |
| `Orbit Station` | Orbit wallet station canister |

### Adding New Categories

If you encounter a pattern that doesn't fit existing categories:
1. Choose a clear, concise name
2. Add it to the table above with a description
3. Use it consistently going forward

**Good category names:**
- Descriptive of what the canister IS or WHY it's unidentifiable
- Short (1-3 words)
- Consistent capitalization

**Examples of when to add a new category:**
- You find 5+ canisters of the same unidentifiable type
- A new proxy/infrastructure pattern emerges
- A specific error condition is common

---

## Research Ranges

| Range | Ranks | Assigned To | Status |
|-------|-------|-------------|--------|
| 1 | 1-100 | | |
| 2 | 101-200 | | |
| 3 | 201-300 | | |
| 4 | 301-400 | | |
| 5 | 401-500 | | |
| 6 | 501-600 | | |
| 7 | 601-700 | | |
| 8 | 701-800 | | |
| 9 | 801-900 | | |
| 10 | 901-1000 | | |
| ... | ... | | |

---

## Example Research Session

**Canister:** `abc12-def34-xxxxx-cai` (rank #47, project = null)

### 1. Get Candid
```bash
$ dfx canister --network ic metadata abc12-def34-xxxxx-cai candid:service

type Account = record { owner : principal; subaccount : opt blob };
type Tokens = nat;
...
service : {
  icrc1_name : () -> (text) query;
  icrc1_symbol : () -> (text) query;
  icrc1_transfer : (TransferArgs) -> (TransferResult);
  ...
}
```
→ This is an ICRC-1 token ledger!

### 2. Query token info
```bash
$ dfx canister --network ic call abc12-def34-xxxxx-cai icrc1_name '()'
("SuperToken")

$ dfx canister --network ic call abc12-def34-xxxxx-cai icrc1_symbol '()'
("SUPER")
```
→ It's the SUPER token ledger!

### 3. Set project
```bash
$ dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_project '(principal "abc12-def34-xxxxx-cai", opt "SUPER Token")'
()
```

### 4. Move to next unknown

---

## Quick Reference

```bash
# Backend canister (for set_project)
vohji-riaaa-aaaac-babxq-cai

# Get Candid
dfx canister --network ic metadata <CID> candid:service 2>&1

# Token info
dfx canister --network ic call <CID> icrc1_name '()'
dfx canister --network ic call <CID> icrc1_symbol '()'

# Index → Ledger
dfx canister --network ic call <CID> ledger_id '()'

# Set identified project
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_project '(principal "<CID>", opt "<NAME>")'

# Mark as unidentified (vetted)
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_project '(principal "<CID>", opt "Unidentified")'
```

---

## Success Criteria

Your range is complete when:
- [ ] Every canister in your range has `project != null`
- [ ] Unknown canisters are marked as `"Unidentified"` (not left as null)
- [ ] Identified projects have accurate, consistent naming

---

## Notes

- **Take your time.** Manual research catches things automation misses.
- **Be curious.** Call methods, read the Candid, check the dashboard.
- **When in doubt, mark as Unidentified.** We can always update later if someone identifies it.
- **Consistent naming matters.** Check existing projects for naming conventions.
