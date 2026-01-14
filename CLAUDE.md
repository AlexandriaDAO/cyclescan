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

## GitHub Actions

`.github/workflows/collect-snapshots.yml` runs hourly at :05:
1. Collect cycle balances from ~2900 canisters
2. Commit updated snapshots.json to repo

That's it - no deployment step. Frontend reads from GitHub directly.

## Key Canister IDs

| Purpose | ID |
|---------|-----|
| ninegua blackhole | `e3mmv-5qaaa-aaaah-aadma-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |
| SNS-W (SNS registry) | `qaa6y-5yaaa-aaaaa-aaafa-cai` |
