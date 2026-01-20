# CycleScan

Cycles burn leaderboard for ICP. Like CoinGecko, but for cycle consumption.

## Architecture

**Static data architecture** - no backend canister, no redeployment needed for data updates:

1. **GitHub Actions** runs hourly to collect cycle balances and commit to repo
2. **Static JSON files** store snapshots (7 days of hourly data)
3. **Frontend canister** fetches data directly from GitHub raw URLs at runtime

Data flow:
- Collection script queries IC canisters using **anonymous principal** (no secrets)
- Commits updated `data/live/snapshots.json` to GitHub
- Frontend fetches from `https://raw.githubusercontent.com/AlexandriaDAO/cyclescan/master/data/...`
- No redeployment required - data updates are live immediately

## Canister

| Canister | ID |
|----------|-----|
| Frontend | `xknwi-uaaaa-aaaak-qu4oq-cai` |

- Live: https://xknwi-uaaaa-aaaak-qu4oq-cai.icp0.io/

## Development

**Deploy frontend (only needed for code changes, not data updates):**
```bash
./scripts/deploy.sh
```

**Run data collection locally:**
```bash
cd scripts && npm run collect
```

Uses `daopad` identity for frontend deployment only.

## Data Files

| File | Purpose |
|------|---------|
| `data/live/snapshots.json` | Hourly cycle balances (auto-updated by GitHub Actions) |
| `data/archive/canisters_backup.json` | Canister registry |
| `data/archive/projects_backup.json` | Project metadata |

## Collection Script

`scripts/collect_snapshots.mjs` queries canisters via:

- **Blackhole**: `canister_status(canister_id)` - one call per canister
- **SNS Root**: `get_sns_canisters_summary()` - one call returns all SNS canisters

Collects **all canisters** in the registry regardless of `valid` flag. Uses **anonymous principal** - no identity or secrets needed.

## Burn Rate Calculations

**Canister types** (set via `valid` field in `canisters_backup.json`):
- `valid: true` (default) - Normal cycle-burning canister
- `valid: false` - Cycle transfer canister (primary purpose is moving cycles, not burning)

**"Include cycle transfers" toggle** (default: OFF):
- **OFF**: Excludes `valid: false` canisters from all calculations (project totals, network burn, sparklines)
- **ON**: Includes all canisters in calculations

**Inferred burn values**: For normal canisters (`valid: true`) that receive top-ups, we can't directly measure burn during those intervals since balance increased. Instead, we calculate the average burn rate from non-top-up intervals and apply it to top-up intervals. This gives accurate burn estimates for canisters that burn cycles but also receive periodic top-ups.

Key files:
- `regression.ts` - Burn rate calculation with top-up detection and inferred values
- `+page.svelte` - Toggle logic and adjusted project calculations
- `data.ts` - Data loading and sparkline aggregation with transfer canister filtering

## API-Sourced Projects

Some projects (like FunnAI) have complex cycle flows that make per-canister tracking inaccurate. These projects provide their own API for aggregate burn data.

**How it works:**
1. Project is removed from `canisters_backup.json` (no individual canisters tracked)
2. Project metadata in `projects_backup.json` has an `api_source` field
3. Collection script queries the project's API and stores aggregate data as `{projectname}-aggregate`
4. Burn rates from APIs are stored directly in `burn_rates` field (not calculated from balance diffs)
5. Frontend shows explanatory notice instead of canister list when expanded

**Adding a new API-sourced project:**

1. Add `api_source` to `projects_backup.json`:
```json
{
  "name": "ProjectName",
  "api_source": {
    "canister_id": "xxxxx-xxxxx-cai",
    "method": "getMetrics",
    "description": "Short explanation shown to users",
    "details": "Longer explanation of why per-canister tracking doesn't work"
  }
}
```

2. Add query function in `collect_snapshots.mjs` (see `queryFunnaiApi` as example)

3. Store both balance and burn rate:
```javascript
finalBalances[AGGREGATE_ID] = data.totalCycles;
burnRates[AGGREGATE_ID] = data.hourlyBurnRate;
```

4. Frontend automatically detects `api_source` projects and handles them appropriately

## GitHub Actions

`.github/workflows/collect-snapshots.yml` runs hourly at :05:
1. Collect cycle balances from ~2900 canisters
2. Commit updated snapshots.json to repo

That's it - no deployment step. Frontend reads from GitHub directly.

## Avoiding Merge Conflicts

**The problem:** `data/live/snapshots.json` is a "hot file" - it's auto-updated hourly by GitHub Actions. If you're working locally and the hourly action runs, you'll get merge conflicts when pushing.

**Best practices:**

1. **Separate code from data commits:**
   - Push code changes first (scripts, frontend, registry files) - these rarely conflict
   - Don't run local collection unless necessary - let GitHub Actions handle data updates
   - The next hourly run will use your updated code automatically

2. **If you must update snapshots.json locally:**
   - Push immediately after the hourly run (e.g., at :10) to maximize time before next conflict
   - Or run collection AND push in quick succession

3. **When conflicts happen:**
   - For `snapshots.json` conflicts, **don't try to merge** - just re-run collection locally
   - `cd scripts && npm run collect` takes ~5 min but guarantees correct data
   - Then commit and push the fresh snapshot

4. **For major data changes** (like adding API integrations):
   - Push code changes first, verify they work
   - The data will self-correct on the next hourly GitHub Actions run
   - No need to fight with merge conflicts for data files

## Key Canister IDs

| Purpose | ID |
|---------|-----|
| ninegua blackhole | `e3mmv-5qaaa-aaaah-aadma-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |
| SNS-W (SNS registry) | `qaa6y-5yaaa-aaaaa-aaafa-cai` |

## Project Metadata Structure

### projects_backup.json

Each project can have a `subcategory_descriptions` field with static tooltip descriptions for canister functions:

```json
{
  "name": "Project Name",
  "website": ["https://example.com"],
  "subcategory_descriptions": {
    "Subcategory Name": "One-line description of what this canister type does"
  }
}
```

**Guidelines for descriptions:**
- Keep descriptions static and factual (avoid values that change like burn rates)
- One line explaining the canister's purpose
- Leave blank if function cannot be determined with certainty

### canisters_backup.json

Each canister entry can have a `subcategory` field matching a key in the project's `subcategory_descriptions`:

```json
{
  "canister_id": "xxxxx-xxxxx-xxxxx-xxxxx-cai",
  "project": ["Project Name"],
  "subcategory": "Subcategory Name",
  "valid": true
}
```

## Researching Canister Functions

To identify what a canister does, query it directly using `dfx`. Do NOT rely on web searches - call the canister's methods.

### Step 1: Get Candid Interface

```bash
dfx canister metadata <canister-id> candid:service --network ic
```

This returns the full interface. Look for recognizable patterns:
- `icrc1_*` methods → Token ledger
- `icrc55_*` methods → DeFi Pylon (vectors)
- `icrc3_*` methods → Archive canister
- `icrc45_*` methods → Price aggregator
- `dex_swap`, `dex_quote` → DEX functionality
- `CreateAsset`, `SetAssetContent` → Asset canister (frontend)
- `get_latest`, `oracle_push` → Price oracle/aggregator

If metadata is restricted (403 error), proceed to Step 2.

### Step 2: Probe Common Methods

Try calling known query methods to deduce the type:

```bash
# Check if it's a DeFi Pylon
dfx canister call <canister-id> icrc55_get_pylon_meta '()' --network ic --query

# Check if it's a token ledger
dfx canister call <canister-id> icrc1_name '()' --network ic --query

# Check if it's an archive
dfx canister call <canister-id> icrc3_get_tip_certificate '()' --network ic --query

# Check if it's a price aggregator
dfx canister call <canister-id> get_latest '()' --network ic --query

# Get basic canister info (module hash)
dfx canister info <canister-id> --network ic
```

### Step 3: Check SNS Root for Registered Canisters

For SNS projects, query the root canister to see registered dapps:

```bash
# List all DeVeFi trading pairs (if project has DeVeFi root)
dfx canister call <devefi-root-id> list_pairs '()' --network ic --query
```

### Common Canister Types

| Type | Key Interface Methods | Description |
|------|----------------------|-------------|
| **DeFi Pylon** | `icrc55_get_pylon_meta`, `icrc55_command` | Hosts automated DeFi vectors (DCA, liquidity, exchange) |
| **DeFi Aggregator** | `get_latest`, `get_pairs`, `oracle_push` | Price oracle aggregating DEX data |
| **Token Ledger** | `icrc1_name`, `icrc1_transfer` | ICRC-1 compliant token |
| **Archive** | `icrc3_get_blocks`, `icrc3_get_tip_certificate` | Stores historical ledger transactions |
| **Asset Canister** | `http_request`, asset batch operations | Frontend hosting |
| **Neuron Pylon** | `icrc55_get_pylon_meta` with neuron modules | ICP/SNS neuron staking automation |
| **Ledger Deployer** | `install`, `upgrade`, `get_account` | Factory for deploying ICRC ledgers |
| **Cycles Relay** | `mint`, `get_queue`, `stats` | Wrapped cycles distribution |

### Example: Neutrinite Canisters

| Canister | Subcategory | Description |
|----------|-------------|-------------|
| `togwv-zqaaa-aaaal-qr7aa-cai` | DeFi Pylon | Hosts automated DeFi vectors with built-in DEX swap |
| `7ew52-sqaaa-aaaal-qsrda-cai` | NTC Relay | Processes wrapped cycles minting and canister top-ups |
| `u45jl-liaaa-aaaam-abppa-cai` | DeFi Aggregator | Price oracle aggregating data from DEXes |
| `fbysu-tqaaa-aaaaq-aacga-cai` | NTN Archive | ICRC-3 archive for NTN token transactions |
| `wxer6-3yaaa-aaaal-qjnua-cai` | DeVeFi Root | Factory that creates DeFi Vector trading pairs |
| `6jvpj-sqaaa-aaaaj-azwnq-cai` | Neuron Pylon | ICP/SNS neuron staking and NTC minting |
| `toj6n-haaaa-aaaal-qdika-cai` | Ledger Deployer | Factory for deploying ICRC-1 token ledgers |
| `nzsmr-6iaaa-aaaal-qsnea-cai` | Liquid Staking | Manages neuron balance and mint ratio |
| `eqtcs-jiaaa-aaaal-qdmia-cai` | Asset Canister | Frontend hosting |
| `3s7ne-diaaa-aaaam-ab24a-cai` | Asset Canister | Frontend hosting |
