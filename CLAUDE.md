# CycleScan

Cycles burn leaderboard for ICP. Like CoinGecko, but for cycle consumption.

## Architecture

**Static data architecture** - no backend canister needed:

1. **GitHub Actions** runs hourly to collect cycle balances
2. **Static JSON files** store snapshots (7 days of hourly data)
3. **Frontend canister** serves everything as static assets

## Canister

| Canister | ID |
|----------|-----|
| Frontend | `xknwi-uaaaa-aaaak-qu4oq-cai` |

- Live: https://xknwi-uaaaa-aaaak-qu4oq-cai.icp0.io/

## Development

**Deploy frontend:**
```bash
./scripts/deploy.sh
```

**Run data collection locally:**
```bash
cd scripts && npm run collect
```

Uses `daopad` identity for deployment, `cyclescan` identity for GitHub Actions.

## Data Files

| File | Purpose |
|------|---------|
| `data/live/snapshots.json` | Hourly cycle balances (auto-updated) |
| `data/archive/canisters_backup.json` | Canister registry |
| `data/archive/projects_backup.json` | Project metadata |

## Collection Script

`scripts/collect_snapshots.mjs` queries canisters via:

- **Blackhole**: `canister_status(canister_id)` - one call per canister
- **SNS Root**: `get_sns_canisters_summary()` - one call returns all SNS canisters

Uses anonymous identity (no keys needed for collection).

## GitHub Actions

`.github/workflows/collect-snapshots.yml` runs hourly at :05:
1. Collect cycle balances from ~2900 canisters
2. Commit updated snapshots.json
3. Deploy frontend with new data

## Key Canister IDs

| Purpose | ID |
|---------|-----|
| ninegua blackhole | `e3mmv-5qaaa-aaaah-aadma-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |
| SNS-W (SNS registry) | `qaa6y-5yaaa-aaaaa-aaafa-cai` |
