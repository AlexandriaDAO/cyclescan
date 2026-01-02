# CycleScan GitHub Static Architecture Plan

## Overview

Replace the backend canister with a pure static approach:
- **GitHub Actions** collects canister balances hourly (free query calls)
- **Static JSON** stores all data in the repo
- **Frontend canister** serves the JSON files directly
- **Zero cycles burned** for data collection and storage

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     GitHub Actions (Hourly)                      │
│                                                                  │
│  1. Query 3,000+ canisters via HTTPS (free)                     │
│  2. Update data/live/snapshots.json                             │
│  3. Commit and push                                              │
│  4. Trigger frontend redeploy (optional)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Git Repository                                │
│                                                                  │
│  data/backup/canisters_backup.json  (canister registry)         │
│  data/backup/projects_backup.json   (project metadata)          │
│  data/live/snapshots.json           (live balance data)         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Frontend Canister (Static)                       │
│                                                                  │
│  Serves JSON files directly, frontend JS calculates burns       │
└─────────────────────────────────────────────────────────────────┘
```

## Data Structure

### `data/live/snapshots.json`

```json
{
  "updated": 1704067200000,
  "snapshot_count": 168,
  "canisters": [
    {
      "id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
      "balance": 50000000000000,
      "balances": {
        "1h": 50100000000000,
        "24h": 52000000000000,
        "7d": 60000000000000
      }
    }
  ]
}
```

### `data/backup/canisters_backup.json` (existing)

Contains canister registry with proxy info. No changes needed.

### `data/backup/projects_backup.json` (existing)

Contains project names and metadata. No changes needed.

## Implementation Steps

### Phase 1: Setup (Day 1)

#### 1.1 Create new dfx identity for GitHub Actions

```bash
# Create a new identity specifically for GitHub Actions
dfx identity new cyclescan-github
dfx identity use cyclescan-github

# Export the private key
dfx identity export cyclescan-github > cyclescan-github.pem

# Get the principal (for reference)
dfx identity get-principal
# Save this principal for future reference

# Switch back to your main identity
dfx identity use daopad
```

#### 1.2 Convert PEM to secret format

```bash
# Extract the raw private key for GitHub secrets
# Option A: Use the full PEM content
cat cyclescan-github.pem | base64 -w 0 > github_key_base64.txt

# Store content of github_key_base64.txt as GitHub secret
# Then DELETE these files - don't commit them
rm cyclescan-github.pem github_key_base64.txt
```

#### 1.3 Add GitHub repository secrets

Go to: Repository → Settings → Secrets and variables → Actions

Add secret:
- Name: `IC_IDENTITY_PEM_B64`
- Value: (the base64 encoded PEM content)

### Phase 2: Collection Script (Day 1-2)

#### 2.1 Create package.json for scripts

```json
// scripts/package.json
{
  "name": "cyclescan-collector",
  "type": "module",
  "scripts": {
    "collect": "node collect_snapshots.mjs"
  },
  "dependencies": {
    "@dfinity/agent": "^1.0.0",
    "@dfinity/principal": "^1.0.0",
    "@dfinity/identity-secp256k1": "^1.0.0"
  }
}
```

#### 2.2 Create collection script

```javascript
// scripts/collect_snapshots.mjs
import { HttpAgent, Actor } from '@dfinity/agent';
import { Secp256k1KeyIdentity } from '@dfinity/identity-secp256k1';
import { Principal } from '@dfinity/principal';
import { readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DATA_DIR = join(__dirname, '..', 'data');

// Blackhole canister for querying canister_status
const BLACKHOLE_ID = 'e3mmv-5qaaa-aaaah-aadma-cai';

// IDL for blackhole canister_status
const blackholeIdl = ({ IDL }) => {
  return IDL.Service({
    canister_status: IDL.Func(
      [IDL.Record({ canister_id: IDL.Principal })],
      [IDL.Record({
        cycles: IDL.Nat,
        status: IDL.Variant({ running: IDL.Null, stopping: IDL.Null, stopped: IDL.Null }),
        memory_size: IDL.Nat,
        // ... other fields we don't need
      })],
      ['query']  // This is a QUERY call - FREE
    ),
  });
};

async function loadCanisters() {
  const canistersPath = join(DATA_DIR, 'backup', 'canisters_backup.json');
  const data = JSON.parse(readFileSync(canistersPath, 'utf-8'));
  return data; // Array of { canister_id, proxy_id, proxy_type, ... }
}

async function loadExistingSnapshots() {
  const snapshotsPath = join(DATA_DIR, 'live', 'snapshots.json');
  try {
    return JSON.parse(readFileSync(snapshotsPath, 'utf-8'));
  } catch {
    return { updated: 0, snapshot_count: 0, canisters: [] };
  }
}

async function queryBlackholeBalance(agent, canisterId) {
  try {
    const actor = Actor.createActor(blackholeIdl, {
      agent,
      canisterId: BLACKHOLE_ID,
    });
    const result = await actor.canister_status({ canister_id: Principal.fromText(canisterId) });
    return BigInt(result.cycles).toString();
  } catch (e) {
    return null;
  }
}

async function querySnsBalance(agent, snsRootId, canisterId) {
  // SNS root returns all canisters in one call
  // Implementation depends on SNS IDL
  // For now, fall back to blackhole if available
  return null;
}

async function main() {
  console.log('Starting CycleScan collection...');

  // Create agent (no identity needed for queries)
  const agent = new HttpAgent({ host: 'https://ic0.app' });

  // Load canister registry
  const canisters = await loadCanisters();
  console.log(`Loaded ${canisters.length} canisters from registry`);

  // Load existing snapshot data
  const existing = await loadExistingSnapshots();
  const existingMap = new Map(existing.canisters.map(c => [c.id, c]));

  // Query all balances in parallel (batched to avoid overwhelming)
  const BATCH_SIZE = 100;
  const results = new Map();

  for (let i = 0; i < canisters.length; i += BATCH_SIZE) {
    const batch = canisters.slice(i, i + BATCH_SIZE);
    console.log(`Querying batch ${i / BATCH_SIZE + 1}/${Math.ceil(canisters.length / BATCH_SIZE)}...`);

    const batchResults = await Promise.all(
      batch.map(async (c) => {
        const balance = await queryBlackholeBalance(agent, c.canister_id);
        return { id: c.canister_id, balance };
      })
    );

    batchResults.forEach(r => {
      if (r.balance !== null) {
        results.set(r.id, r.balance);
      }
    });
  }

  console.log(`Successfully queried ${results.size} canisters`);

  // Build new snapshot data
  const now = Date.now();
  const newCanisters = canisters.map(c => {
    const currentBalance = results.get(c.canister_id);
    const prev = existingMap.get(c.canister_id);

    if (currentBalance === null && !prev) {
      return null; // No data at all
    }

    return {
      id: c.canister_id,
      balance: currentBalance || (prev?.balance || null),
      balances: {
        '1h': prev?.balance || null,
        '24h': prev?.balances?.['1h'] || prev?.balances?.['24h'] || null,
        '7d': prev?.balances?.['24h'] || prev?.balances?.['7d'] || null,
      }
    };
  }).filter(Boolean);

  // Note: The above shift logic is simplified.
  // In practice, you'd track timestamps and only shift when appropriate.

  const output = {
    updated: now,
    snapshot_count: existing.snapshot_count + 1,
    canisters: newCanisters,
  };

  // Write output
  const outputPath = join(DATA_DIR, 'live', 'snapshots.json');
  writeFileSync(outputPath, JSON.stringify(output, null, 2));
  console.log(`Wrote ${newCanisters.length} canisters to ${outputPath}`);
}

main().catch(console.error);
```

### Phase 3: GitHub Workflow (Day 2)

#### 3.1 Create workflow file

```yaml
# .github/workflows/collect-snapshots.yml
name: Collect Cycle Snapshots

on:
  schedule:
    # Run every hour at minute 5 (avoid exact hour congestion)
    - cron: '5 * * * *'
  workflow_dispatch:
    # Allow manual trigger for testing

jobs:
  collect:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: scripts/package-lock.json

      - name: Install dependencies
        working-directory: scripts
        run: npm ci

      - name: Create live data directory
        run: mkdir -p data/live

      - name: Collect snapshots
        working-directory: scripts
        run: node collect_snapshots.mjs

      - name: Commit and push
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"

          git add data/live/snapshots.json

          # Only commit if there are changes
          if git diff --staged --quiet; then
            echo "No changes to commit"
          else
            git commit -m "📊 Hourly snapshot $(date -u +%Y-%m-%d_%H:%M)"
            git push
          fi
```

### Phase 4: Frontend Changes (Day 2-3)

#### 4.1 Update frontend to fetch static JSON

```svelte
<!-- src/cyclescan_frontend/src/routes/+page.svelte -->
<script>
  import { onMount } from 'svelte';

  let canisters = [];
  let projects = {};
  let loading = true;
  let lastUpdated = null;

  onMount(async () => {
    // Fetch all static data
    const [snapshotsRes, canistersRes, projectsRes] = await Promise.all([
      fetch('/data/live/snapshots.json'),
      fetch('/data/backup/canisters_backup.json'),
      fetch('/data/backup/projects_backup.json'),
    ]);

    const snapshots = await snapshotsRes.json();
    const canisterRegistry = await canistersRes.json();
    const projectList = await projectsRes.json();

    // Build project lookup
    projectList.forEach(p => {
      projects[p.canister_id] = p.project_name;
    });

    // Merge data and calculate burns
    canisters = snapshots.canisters.map(s => {
      const registry = canisterRegistry.find(c => c.canister_id === s.id);
      return {
        id: s.id,
        project: projects[s.id] || registry?.project_name || null,
        balance: BigInt(s.balance),
        burn_1h: calculateBurn(s.balance, s.balances['1h']),
        burn_24h: calculateBurn(s.balance, s.balances['24h']),
        burn_7d: calculateBurn(s.balance, s.balances['7d']),
      };
    });

    // Sort by 24h burn (descending)
    canisters.sort((a, b) => (b.burn_24h || 0n) - (a.burn_24h || 0n));

    lastUpdated = new Date(snapshots.updated);
    loading = false;
  });

  function calculateBurn(current, previous) {
    if (!current || !previous) return null;
    const c = BigInt(current);
    const p = BigInt(previous);
    return p > c ? p - c : 0n;
  }
</script>
```

#### 4.2 Update dfx.json to include data files

```json
{
  "canisters": {
    "cyclescan_frontend": {
      "type": "assets",
      "source": [
        "src/cyclescan_frontend/dist",
        "data"
      ]
    }
  }
}
```

Or symlink/copy data files into the frontend build.

### Phase 5: Testing (Day 3)

#### 5.1 Test collection locally

```bash
cd scripts
npm install
node collect_snapshots.mjs
```

Verify `data/live/snapshots.json` is created correctly.

#### 5.2 Test GitHub Action manually

1. Go to Actions tab in GitHub
2. Select "Collect Cycle Snapshots"
3. Click "Run workflow"
4. Verify it commits successfully

#### 5.3 Test frontend locally

```bash
npm run build
# Serve and verify data loads correctly
```

### Phase 6: Deploy (Day 3-4)

#### 6.1 Deploy frontend with static data

```bash
./scripts/deploy.sh
```

#### 6.2 Verify everything works

1. Visit the frontend
2. Check data loads correctly
3. Wait for next hourly run
4. Verify data updates

### Phase 7: Cleanup (Day 4+)

#### 7.1 Stop the backend timer (optional)

If the backend has an automatic timer, disable it to stop burning cycles:

```bash
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai stop_timer '()'
```

Or just let it run out of cycles naturally.

#### 7.2 Consider archiving the backend

The backend still holds historical data. You may want to:
- Export all historical snapshots before it dies
- Keep it around as a backup
- Or just let it be

## File Structure After Implementation

```
cyclescan/
├── .github/
│   └── workflows/
│       └── collect-snapshots.yml    # Hourly cron job
├── data/
│   ├── backup/
│   │   ├── canisters_backup.json    # Canister registry (existing)
│   │   └── projects_backup.json     # Project names (existing)
│   └── live/
│       └── snapshots.json           # Live balance data (generated)
├── scripts/
│   ├── package.json
│   └── collect_snapshots.mjs        # Collection script
└── src/
    └── cyclescan_frontend/          # Frontend (updated)
```

## Cost Comparison

| Component | Old Architecture | New Architecture |
|-----------|-----------------|------------------|
| Backend canister cycles | ~50T/day | 0 |
| GitHub Actions | $0 | $0 (free tier) |
| Frontend canister | ~1-2T/day | ~1-2T/day |
| **Total** | **~50T/day** | **~1-2T/day** |

**Savings: ~98%**

## Rollback Plan

If this approach doesn't work:

1. Backend canister is still running (untouched)
2. Re-enable frontend to call backend API
3. Top up backend with cycles
4. Disable GitHub Action

## Open Questions

1. **Historical data**: Do we need charts with more than 4 data points?
   - If yes, consider storing more history in snapshots.json

2. **SNS canisters**: The collection script above only handles Blackhole proxy.
   - Need to add SNS root querying logic

3. **Failed queries**: How to handle canisters that consistently fail?
   - Consider removing them from the registry after N failures

4. **Git repo size**: snapshots.json will be committed hourly.
   - ~500KB × 24 × 365 = ~4GB/year in git history
   - Consider squashing history periodically, or using git LFS

## Next Steps

1. [ ] Create `cyclescan-github` dfx identity
2. [ ] Add identity to GitHub secrets
3. [ ] Create `scripts/` directory with collection script
4. [ ] Create `.github/workflows/collect-snapshots.yml`
5. [ ] Test locally
6. [ ] Test GitHub Action manually
7. [ ] Update frontend to use static JSON
8. [ ] Deploy and verify
9. [ ] Celebrate saving 98% on cycles
