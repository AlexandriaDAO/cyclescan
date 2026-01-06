# CycleScan

Cycles burn leaderboard for ICP. Like CoinGecko, but for cycle consumption.

**Live:** https://xknwi-uaaaa-aaaak-qu4oq-cai.icp0.io/

## How it Works

1. **GitHub Actions** runs hourly to collect cycle balances from ~2900 tracked canisters
2. Data is committed to `data/live/snapshots.json` (7 days of hourly history)
3. Frontend fetches data directly from GitHub - no redeployment needed for updates

## Development

See [CLAUDE.md](./CLAUDE.md) for detailed architecture and development instructions.

```bash
# Deploy frontend (only needed for code changes)
./scripts/deploy.sh

# Run data collection locally
cd scripts && npm run collect
```

## License

MIT
