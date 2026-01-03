# GitHub Direct Fetch Architecture

## Overview

Simplify the data pipeline by having the frontend fetch live data directly from GitHub instead of bundling it into the frontend canister.

## Current Architecture

```
GitHub Actions (hourly)
    ↓
1. Collect cycle balances from IC
    ↓
2. Commit snapshots.json to git
    ↓
3. Build frontend
    ↓
4. Deploy frontend to IC canister
```

**Problems:**
- Requires `cyclescan` identity with deploy permissions
- Requires `DFX_IDENTITY_PEM_B64` secret in GitHub
- Deploy step takes ~2 minutes
- More moving parts = more failure points

## Proposed Architecture

```
GitHub Actions (hourly)
    ↓
1. Collect cycle balances from IC
    ↓
2. Commit snapshots.json to git
    ↓
Done.

Frontend (on page load)
    ↓
Fetches from raw.githubusercontent.com
```

**Benefits:**
- No deploy step needed
- No identity/secret management
- Faster workflow (~10 min vs ~12 min)
- Simpler, fewer failure points
- Frontend always gets latest data

## Data URLs

The frontend will fetch from GitHub raw URLs:

| File | URL |
|------|-----|
| Snapshots | `https://raw.githubusercontent.com/AlexandriaDAO/cyclescan/master/data/live/snapshots.json` |
| Canisters | `https://raw.githubusercontent.com/AlexandriaDAO/cyclescan/master/data/archive/canisters_backup.json` |
| Projects | `https://raw.githubusercontent.com/AlexandriaDAO/cyclescan/master/data/archive/projects_backup.json` |

## Implementation

### 1. Update Frontend (`src/lib/data.ts`)

Change fetch URLs from relative paths to GitHub raw URLs:

```typescript
const GITHUB_RAW = 'https://raw.githubusercontent.com/AlexandriaDAO/cyclescan/master';

const [snapshotsRes, canistersRes, projectsRes] = await Promise.all([
  fetch(`${GITHUB_RAW}/data/live/snapshots.json`),
  fetch(`${GITHUB_RAW}/data/archive/canisters_backup.json`),
  fetch(`${GITHUB_RAW}/data/archive/projects_backup.json`),
]);
```

### 2. Simplify Workflow (`.github/workflows/collect-snapshots.yml`)

Remove all deploy-related steps:

```yaml
name: Collect Cycle Snapshots

on:
  schedule:
    - cron: '5 * * * *'
  workflow_dispatch:

permissions:
  contents: write

jobs:
  collect:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: scripts/package-lock.json

      - name: Install dependencies
        working-directory: scripts
        run: npm ci

      - name: Collect snapshots
        working-directory: scripts
        run: node collect_snapshots.mjs

      - name: Commit changes
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add data/live/snapshots.json
          if ! git diff --staged --quiet; then
            git commit -m "Hourly snapshot $(date -u +%Y-%m-%d_%H:%M)"
            git pull --rebase origin master
            git push
          fi
```

### 3. Deploy Frontend Once

Deploy the updated frontend with GitHub fetch URLs:

```bash
./scripts/deploy.sh
```

After this single deploy, no further deploys needed for data updates.

### 4. Clean Up

- Remove `DFX_IDENTITY_PEM_B64` secret from GitHub repo settings
- Optionally remove `cyclescan` identity from dfx
- Remove `cyclescan` as controller of frontend canister (optional)

## Considerations

### CORS
GitHub's `raw.githubusercontent.com` allows cross-origin requests, so no CORS issues.

### Caching
GitHub CDN caches raw files for ~5 minutes. This is acceptable for hourly data.
Add cache-busting if needed: `?t=${Date.now()}`

### Reliability
GitHub has 99.9%+ uptime. If GitHub is down, the IC frontend itself is likely also unreachable.

### Fallback (Optional)
Keep static data in the canister as a fallback if GitHub fetch fails:

```typescript
try {
  // Try GitHub first
  const data = await fetch(GITHUB_RAW + '/data/live/snapshots.json');
} catch {
  // Fall back to bundled data
  const data = await fetch('/live/snapshots.json');
}
```

## Migration Checklist

- [ ] Update `src/lib/data.ts` with GitHub URLs
- [ ] Simplify `.github/workflows/collect-snapshots.yml`
- [ ] Deploy frontend once with `./scripts/deploy.sh`
- [ ] Verify frontend loads data from GitHub
- [ ] Remove `DFX_IDENTITY_PEM_B64` secret from GitHub
- [ ] Update `CLAUDE.md` documentation
