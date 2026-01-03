# CycleScan GitHub Static Architecture Plan

## Overview

Replace the backend canister with a pure static approach:
- **GitHub Actions** collects canister balances hourly (free query calls)
- **Static JSON** stores all data in the repo (168 hours of history)
- **Frontend canister** serves the JSON files directly
- **Auto-redeploy** frontend after each data collection
- **Zero cycles burned** for data collection and storage

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     GitHub Actions (Hourly)                      │
│                                                                  │
│  1. Query 3,140 canisters via HTTPS (free query calls)          │
│     - 2,637 Blackhole canisters → canister_status per canister  │
│     - 503 SNS canisters → get_sns_canisters_summary (41 calls)  │
│  2. Update data/live/snapshots.json (append to 168-hour history)│
│  3. Commit and push                                              │
│  4. Auto-deploy frontend canister with updated data             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Git Repository                                │
│                                                                  │
│  data/backup/canisters_backup.json  (canister registry)         │
│  data/backup/projects_backup.json   (project metadata)          │
│  data/live/snapshots.json           (168 hours of balances)     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Frontend Canister (Static)                       │
│                                                                  │
│  Serves JSON files directly, frontend JS calculates burns       │
│  from historical snapshots (1h, 24h, 7d comparisons)            │
└─────────────────────────────────────────────────────────────────┘
```

## Data Structure

### `data/live/snapshots.json`

Stores 168 hours (7 days) of hourly snapshots. Each snapshot contains all canister balances at that hour.

```json
{
  "snapshots": [
    {
      "timestamp": 1704067200000,
      "balances": {
        "ryjl3-tyaaa-aaaaa-aaaba-cai": "50000000000000",
        "rkp4c-7iaaa-aaaaa-aaaca-cai": "25000000000000"
      }
    },
    {
      "timestamp": 1704063600000,
      "balances": {
        "ryjl3-tyaaa-aaaaa-aaaba-cai": "50100000000000",
        "rkp4c-7iaaa-aaaaa-aaaca-cai": "25050000000000"
      }
    }
  ]
}
```

**Notes:**
- Array is ordered newest-first (index 0 = current, index 1 = 1h ago, etc.)
- Maximum 168 entries (7 days × 24 hours)
- Balances stored as strings to handle large numbers safely in JSON
- Frontend calculates burn rates by comparing snapshots at appropriate offsets:
  - 1h burn: `snapshots[1].balances[id] - snapshots[0].balances[id]`
  - 24h burn: `snapshots[24].balances[id] - snapshots[0].balances[id]`
  - 7d burn: `snapshots[168].balances[id] - snapshots[0].balances[id]`

**Estimated file size:** ~3-5MB (3,140 canisters × 168 snapshots × ~10 bytes per entry)

### `data/backup/canisters_backup.json` (existing)

Contains canister registry with proxy info. Structure:
```json
[
  {
    "canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
    "project": ["ICP Ledger"],
    "proxy_id": "r7inp-6aaaa-aaaaa-aaabq-cai",
    "proxy_type": { "Blackhole": null },
    "valid": true
  },
  {
    "canister_id": "zxeu2-...",
    "project": ["OpenChat"],
    "proxy_id": "3e3x2-xyaaa-aaaaq-aaalq-cai",
    "proxy_type": { "SnsRoot": null },
    "valid": true
  }
]
```

### `data/backup/projects_backup.json` (existing)

Contains project names and metadata. No changes needed.

## Implementation Steps

### Phase 1: Setup

#### 1.1 Create deploy identity for GitHub Actions

Create a dedicated dfx identity for auto-deploying the frontend canister.

```bash
# Create a new identity specifically for GitHub Actions deploys
dfx identity new cyclescan-deploy
dfx identity use cyclescan-deploy

# Get the principal
dfx identity get-principal
# Example output: 2vxsx-fae... (save this!)

# Export the private key
dfx identity export cyclescan-deploy > cyclescan-deploy.pem

# Switch back to your main identity
dfx identity use daopad
```

#### 1.2 Add deploy identity as frontend controller

```bash
# Add the new identity as a controller of the frontend canister
dfx canister --network ic update-settings cyclescan_frontend \
  --add-controller <PRINCIPAL_FROM_STEP_1.1>
```

#### 1.3 Convert PEM to GitHub secret format

```bash
# Base64 encode the PEM file for GitHub secrets
cat cyclescan-deploy.pem | base64 -w 0 > github_key_base64.txt

# Copy the contents of github_key_base64.txt

# IMPORTANT: Delete these files - never commit them!
rm cyclescan-deploy.pem github_key_base64.txt
```

#### 1.4 Add GitHub repository secrets

Go to: Repository → Settings → Secrets and variables → Actions

Add these secrets:
| Name | Value |
|------|-------|
| `DFX_IDENTITY_PEM_B64` | The base64-encoded PEM content from step 1.3 |

**Note:** Query calls (collecting balances) don't require authentication - they're anonymous.
The identity is only needed for deploying the frontend canister.

### Phase 2: Collection Script

#### 2.1 Create package.json for scripts

```json
{
  "name": "cyclescan-collector",
  "type": "module",
  "scripts": {
    "collect": "node collect_snapshots.mjs"
  },
  "dependencies": {
    "@dfinity/agent": "^2.0.0",
    "@dfinity/principal": "^2.0.0",
    "@dfinity/candid": "^2.0.0"
  }
}
```

#### 2.2 Create collection script

```javascript
// scripts/collect_snapshots.mjs
import { HttpAgent, Actor } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { IDL } from '@dfinity/candid';
import { readFileSync, writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DATA_DIR = join(__dirname, '..', 'data');
const MAX_SNAPSHOTS = 168; // 7 days of hourly snapshots
const BATCH_SIZE = 50; // Concurrent requests per batch

// ============================================================================
// IDL Definitions
// ============================================================================

// Blackhole canister_status interface (works for any blackhole controller)
const blackholeIdl = ({ IDL }) => {
  return IDL.Service({
    canister_status: IDL.Func(
      [IDL.Record({ canister_id: IDL.Principal })],
      [IDL.Record({
        cycles: IDL.Nat,
        status: IDL.Variant({
          running: IDL.Null,
          stopping: IDL.Null,
          stopped: IDL.Null
        }),
        memory_size: IDL.Nat,
        module_hash: IDL.Opt(IDL.Vec(IDL.Nat8)),
        settings: IDL.Record({
          controllers: IDL.Vec(IDL.Principal),
          compute_allocation: IDL.Nat,
          memory_allocation: IDL.Nat,
          freezing_threshold: IDL.Nat,
        }),
      })],
      ['query']
    ),
  });
};

// SNS Root get_sns_canisters_summary interface
const snsRootIdl = ({ IDL }) => {
  const CanisterStatusResult = IDL.Record({
    cycles: IDL.Nat,
  });
  const CanisterSummary = IDL.Record({
    canister_id: IDL.Opt(IDL.Principal),
    status: IDL.Opt(CanisterStatusResult),
  });
  return IDL.Service({
    get_sns_canisters_summary: IDL.Func(
      [IDL.Record({})],
      [IDL.Record({
        root: IDL.Opt(CanisterSummary),
        governance: IDL.Opt(CanisterSummary),
        ledger: IDL.Opt(CanisterSummary),
        swap: IDL.Opt(CanisterSummary),
        index: IDL.Opt(CanisterSummary),
        archives: IDL.Vec(CanisterSummary),
        dapps: IDL.Vec(CanisterSummary),
      })],
      ['query']
    ),
  });
};

// ============================================================================
// Data Loading
// ============================================================================

function loadCanisters() {
  const canistersPath = join(DATA_DIR, 'backup', 'canisters_backup.json');
  const data = JSON.parse(readFileSync(canistersPath, 'utf-8'));
  // Filter to only valid canisters
  return data.filter(c => c.valid !== false);
}

function loadExistingSnapshots() {
  const snapshotsPath = join(DATA_DIR, 'live', 'snapshots.json');
  try {
    return JSON.parse(readFileSync(snapshotsPath, 'utf-8'));
  } catch {
    return { snapshots: [] };
  }
}

// ============================================================================
// Query Functions
// ============================================================================

async function queryBlackhole(agent, proxyId, canisterId) {
  try {
    const actor = Actor.createActor(blackholeIdl, {
      agent,
      canisterId: proxyId, // Use the canister's specific proxy (blackhole controller)
    });
    const result = await actor.canister_status({
      canister_id: Principal.fromText(canisterId)
    });
    return result.cycles.toString();
  } catch (e) {
    console.error(`  Failed to query ${canisterId} via ${proxyId}: ${e.message}`);
    return null;
  }
}

async function querySnsRoot(agent, snsRootId) {
  try {
    const actor = Actor.createActor(snsRootIdl, {
      agent,
      canisterId: snsRootId,
    });
    const result = await actor.get_sns_canisters_summary({});

    // Extract all canisters and their cycles
    const balances = new Map();

    const allCanisters = [
      result.root,
      result.governance,
      result.ledger,
      result.swap,
      result.index,
      ...(result.archives || []),
      ...(result.dapps || []),
    ];

    for (const summary of allCanisters) {
      if (summary && summary.canister_id?.[0] && summary.status?.[0]) {
        const id = summary.canister_id[0].toText();
        const cycles = summary.status[0].cycles.toString();
        balances.set(id, cycles);
      }
    }

    return balances;
  } catch (e) {
    console.error(`  Failed to query SNS root ${snsRootId}: ${e.message}`);
    return new Map();
  }
}

// ============================================================================
// Main Collection Logic
// ============================================================================

async function collectBalances(agent, canisters) {
  const results = new Map();

  // Separate canisters by proxy type
  const blackholeCanisters = canisters.filter(c => c.proxy_type?.Blackhole !== undefined);
  const snsCanisters = canisters.filter(c => c.proxy_type?.SnsRoot !== undefined);

  console.log(`Canisters to query:`);
  console.log(`  - Blackhole: ${blackholeCanisters.length}`);
  console.log(`  - SNS Root: ${snsCanisters.length}`);

  // -------------------------------------------------------------------------
  // Query SNS canisters (grouped by SNS root - more efficient)
  // -------------------------------------------------------------------------
  const snsRoots = [...new Set(snsCanisters.map(c => c.proxy_id))];
  console.log(`\nQuerying ${snsRoots.length} SNS roots...`);

  for (const snsRootId of snsRoots) {
    const snsBalances = await querySnsRoot(agent, snsRootId);
    for (const [id, balance] of snsBalances) {
      results.set(id, balance);
    }
    // Small delay between SNS queries
    await new Promise(r => setTimeout(r, 100));
  }
  console.log(`  Got balances for ${results.size} SNS canisters`);

  // -------------------------------------------------------------------------
  // Query Blackhole canisters (batched)
  // -------------------------------------------------------------------------
  console.log(`\nQuerying ${blackholeCanisters.length} blackhole canisters...`);

  for (let i = 0; i < blackholeCanisters.length; i += BATCH_SIZE) {
    const batch = blackholeCanisters.slice(i, i + BATCH_SIZE);
    const batchNum = Math.floor(i / BATCH_SIZE) + 1;
    const totalBatches = Math.ceil(blackholeCanisters.length / BATCH_SIZE);

    process.stdout.write(`  Batch ${batchNum}/${totalBatches}...`);

    const batchResults = await Promise.all(
      batch.map(async (c) => {
        const balance = await queryBlackhole(agent, c.proxy_id, c.canister_id);
        return { id: c.canister_id, balance };
      })
    );

    let successCount = 0;
    for (const { id, balance } of batchResults) {
      if (balance !== null) {
        results.set(id, balance);
        successCount++;
      }
    }
    console.log(` ${successCount}/${batch.length} succeeded`);

    // Small delay between batches
    await new Promise(r => setTimeout(r, 200));
  }

  return results;
}

async function main() {
  console.log('='.repeat(60));
  console.log('CycleScan Collection');
  console.log(`Time: ${new Date().toISOString()}`);
  console.log('='.repeat(60));

  // Create agent (anonymous - no identity needed for queries)
  const agent = new HttpAgent({ host: 'https://icp-api.io' });

  // Load canister registry
  const canisters = loadCanisters();
  console.log(`\nLoaded ${canisters.length} canisters from registry`);

  // Load existing snapshots
  const existing = loadExistingSnapshots();
  console.log(`Existing snapshots: ${existing.snapshots.length}`);

  // Get last known balances (for fallback on failed queries)
  const lastKnownBalances = existing.snapshots[0]?.balances || {};

  // Collect current balances
  const currentBalances = await collectBalances(agent, canisters);

  // Merge with last known values for failed queries
  const finalBalances = {};
  for (const c of canisters) {
    const current = currentBalances.get(c.canister_id);
    if (current !== undefined) {
      finalBalances[c.canister_id] = current;
    } else if (lastKnownBalances[c.canister_id]) {
      // Keep last known value if query failed
      finalBalances[c.canister_id] = lastKnownBalances[c.canister_id];
    }
  }

  console.log(`\nFinal balances: ${Object.keys(finalBalances).length} canisters`);
  console.log(`  - Fresh queries: ${currentBalances.size}`);
  console.log(`  - From last known: ${Object.keys(finalBalances).length - currentBalances.size}`);

  // Create new snapshot
  const newSnapshot = {
    timestamp: Date.now(),
    balances: finalBalances,
  };

  // Prepend to snapshots array, keep only MAX_SNAPSHOTS
  const newSnapshots = [newSnapshot, ...existing.snapshots].slice(0, MAX_SNAPSHOTS);

  // Write output
  mkdirSync(join(DATA_DIR, 'live'), { recursive: true });
  const outputPath = join(DATA_DIR, 'live', 'snapshots.json');
  writeFileSync(outputPath, JSON.stringify({ snapshots: newSnapshots }, null, 2));

  console.log(`\nWrote ${newSnapshots.length} snapshots to ${outputPath}`);
  console.log('='.repeat(60));
}

main().catch(e => {
  console.error('Collection failed:', e);
  process.exit(1);
});
```

### Phase 3: GitHub Workflow

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
  collect-and-deploy:
    runs-on: ubuntu-latest

    steps:
      # -----------------------------------------------------------------------
      # Setup
      # -----------------------------------------------------------------------
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: scripts/package-lock.json

      - name: Install collection dependencies
        working-directory: scripts
        run: npm ci

      # -----------------------------------------------------------------------
      # Collect Data
      # -----------------------------------------------------------------------
      - name: Collect snapshots from IC
        working-directory: scripts
        run: node collect_snapshots.mjs

      # -----------------------------------------------------------------------
      # Commit to Git
      # -----------------------------------------------------------------------
      - name: Commit snapshot data
        id: commit
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"

          git add data/live/snapshots.json

          if git diff --staged --quiet; then
            echo "No changes to commit"
            echo "changed=false" >> $GITHUB_OUTPUT
          else
            git commit -m "📊 Hourly snapshot $(date -u +%Y-%m-%d_%H:%M)"
            git push
            echo "changed=true" >> $GITHUB_OUTPUT
          fi

      # -----------------------------------------------------------------------
      # Deploy Frontend (only if data changed)
      # -----------------------------------------------------------------------
      - name: Install dfx
        if: steps.commit.outputs.changed == 'true'
        uses: dfinity/setup-dfx@main

      - name: Setup deploy identity
        if: steps.commit.outputs.changed == 'true'
        run: |
          # Decode the base64 PEM and import as identity
          echo "${{ secrets.DFX_IDENTITY_PEM_B64 }}" | base64 -d > /tmp/deploy.pem
          dfx identity import --storage-mode=plaintext cyclescan-deploy /tmp/deploy.pem
          dfx identity use cyclescan-deploy
          rm /tmp/deploy.pem

      - name: Build frontend
        if: steps.commit.outputs.changed == 'true'
        run: |
          cd src/cyclescan_frontend
          npm ci
          npm run build

      - name: Deploy frontend to IC
        if: steps.commit.outputs.changed == 'true'
        run: |
          dfx deploy --network ic cyclescan_frontend --no-wallet

      - name: Cleanup identity
        if: always()
        run: |
          dfx identity remove cyclescan-deploy 2>/dev/null || true
```

**Notes:**
- Workflow only deploys if snapshot data actually changed
- Identity is created, used, and removed in the same run (never persisted)
- Uses `--no-wallet` since asset canisters don't need cycles for deployment

### Phase 4: Frontend Changes

#### 4.1 Create data loading utilities

```typescript
// src/cyclescan_frontend/src/lib/data.ts

export interface Snapshot {
  timestamp: number;
  balances: Record<string, string>;
}

export interface SnapshotsData {
  snapshots: Snapshot[];
}

export interface CanisterRegistry {
  canister_id: string;
  project: string[] | null;
  proxy_id: string;
  proxy_type: { Blackhole: null } | { SnsRoot: null };
  valid: boolean;
}

export interface CanisterRow {
  id: string;
  project: string | null;
  balance: bigint;
  burn_1h: bigint | null;
  burn_24h: bigint | null;
  burn_7d: bigint | null;
}

export async function loadData(): Promise<{
  canisters: CanisterRow[];
  lastUpdated: Date;
  snapshotCount: number;
}> {
  const [snapshotsRes, registryRes] = await Promise.all([
    fetch('/data/live/snapshots.json'),
    fetch('/data/backup/canisters_backup.json'),
  ]);

  const snapshotsData: SnapshotsData = await snapshotsRes.json();
  const registry: CanisterRegistry[] = await registryRes.json();

  const { snapshots } = snapshotsData;
  if (snapshots.length === 0) {
    return { canisters: [], lastUpdated: new Date(), snapshotCount: 0 };
  }

  // Build registry lookup
  const registryMap = new Map(registry.map(r => [r.canister_id, r]));

  // Get current balances (index 0)
  const currentSnapshot = snapshots[0];
  const currentBalances = currentSnapshot.balances;

  // Get historical snapshots for comparison
  const snapshot1h = snapshots[1]?.balances || {};    // 1 hour ago
  const snapshot24h = snapshots[24]?.balances || {};  // 24 hours ago
  const snapshot7d = snapshots[167]?.balances || {};  // ~7 days ago (index 167 = 168th entry)

  // Build canister rows
  const canisters: CanisterRow[] = Object.entries(currentBalances).map(([id, balanceStr]) => {
    const reg = registryMap.get(id);
    const balance = BigInt(balanceStr);

    return {
      id,
      project: reg?.project?.[0] || null,
      balance,
      burn_1h: calculateBurn(balance, snapshot1h[id]),
      burn_24h: calculateBurn(balance, snapshot24h[id]),
      burn_7d: calculateBurn(balance, snapshot7d[id]),
    };
  });

  // Sort by 24h burn (descending), nulls last
  canisters.sort((a, b) => {
    if (a.burn_24h === null && b.burn_24h === null) return 0;
    if (a.burn_24h === null) return 1;
    if (b.burn_24h === null) return -1;
    return b.burn_24h > a.burn_24h ? 1 : b.burn_24h < a.burn_24h ? -1 : 0;
  });

  return {
    canisters,
    lastUpdated: new Date(currentSnapshot.timestamp),
    snapshotCount: snapshots.length,
  };
}

function calculateBurn(current: bigint, previousStr: string | undefined): bigint | null {
  if (!previousStr) return null;
  const previous = BigInt(previousStr);
  // Burn = previous - current (positive if cycles decreased)
  return previous > current ? previous - current : 0n;
}
```

#### 4.2 Update frontend page

```svelte
<!-- src/cyclescan_frontend/src/routes/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { loadData, type CanisterRow } from '$lib/data';

  let canisters: CanisterRow[] = [];
  let loading = true;
  let lastUpdated: Date | null = null;
  let snapshotCount = 0;
  let error: string | null = null;

  onMount(async () => {
    try {
      const data = await loadData();
      canisters = data.canisters;
      lastUpdated = data.lastUpdated;
      snapshotCount = data.snapshotCount;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      loading = false;
    }
  });

  function formatCycles(cycles: bigint): string {
    const trillion = 1_000_000_000_000n;
    if (cycles >= trillion) {
      return (Number(cycles) / 1e12).toFixed(2) + 'T';
    }
    const billion = 1_000_000_000n;
    if (cycles >= billion) {
      return (Number(cycles) / 1e9).toFixed(2) + 'B';
    }
    return cycles.toLocaleString();
  }

  function formatBurn(burn: bigint | null): string {
    if (burn === null) return '—';
    if (burn === 0n) return '0';
    return formatCycles(burn);
  }
</script>

{#if loading}
  <p>Loading...</p>
{:else if error}
  <p class="error">{error}</p>
{:else}
  <header>
    <p>Last updated: {lastUpdated?.toLocaleString()}</p>
    <p>History: {snapshotCount} snapshots ({Math.floor(snapshotCount / 24)} days)</p>
  </header>

  <table>
    <thead>
      <tr>
        <th>Project</th>
        <th>Canister</th>
        <th>Balance</th>
        <th>1h Burn</th>
        <th>24h Burn</th>
        <th>7d Burn</th>
      </tr>
    </thead>
    <tbody>
      {#each canisters as canister}
        <tr>
          <td>{canister.project || '—'}</td>
          <td><code>{canister.id}</code></td>
          <td>{formatCycles(canister.balance)}</td>
          <td>{formatBurn(canister.burn_1h)}</td>
          <td>{formatBurn(canister.burn_24h)}</td>
          <td>{formatBurn(canister.burn_7d)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
```

#### 4.3 Update dfx.json to include data files

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

**Note:** The `data/` directory is included as an asset source, so both
`data/backup/*.json` and `data/live/*.json` will be served by the frontend canister.

### Phase 5: Testing

#### 5.1 Test collection locally

```bash
cd scripts
npm install
node collect_snapshots.mjs
```

**Expected output:**
```
============================================================
CycleScan Collection
Time: 2024-01-15T12:05:00.000Z
============================================================

Loaded 3140 canisters from registry
Existing snapshots: 0
Canisters to query:
  - Blackhole: 2637
  - SNS Root: 503

Querying 41 SNS roots...
  Got balances for 503 SNS canisters

Querying 2637 blackhole canisters...
  Batch 1/53... 50/50 succeeded
  Batch 2/53... 50/50 succeeded
  ...

Final balances: 3140 canisters
  - Fresh queries: 3140
  - From last known: 0

Wrote 1 snapshots to /path/to/data/live/snapshots.json
============================================================
```

Verify `data/live/snapshots.json` is created with expected structure.

#### 5.2 Test GitHub Action manually

1. Commit and push the workflow file
2. Go to Actions tab in GitHub
3. Select "Collect Cycle Snapshots"
4. Click "Run workflow" → "Run workflow"
5. Wait for completion (~5-10 minutes)
6. Verify:
   - Snapshot data committed to repo
   - Frontend deployed successfully

#### 5.3 Test frontend locally

```bash
cd src/cyclescan_frontend
npm install
npm run build
npm run preview  # or use any static server
```

Verify:
- Data loads without errors
- Canister list displays correctly
- Burn rates show for canisters with history

### Phase 6: Initial Deploy

#### 6.1 Deploy frontend manually first time

Before enabling the GitHub Action, do one manual deploy:

```bash
# Make sure you're using your main identity
dfx identity use daopad

# Build and deploy
cd src/cyclescan_frontend
npm run build
cd ../..
dfx deploy --network ic cyclescan_frontend
```

#### 6.2 Verify everything works

1. Visit the frontend: `https://<canister-id>.icp0.io`
2. Check data loads correctly
3. Wait for next hourly GitHub Action run
4. Verify data updates and frontend redeploys

### Phase 7: Cleanup (Optional)

#### 7.1 Stop the backend timer

If the backend has an automatic timer burning cycles, disable it:

```bash
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai stop_timer '()'
```

Or just let it run out of cycles naturally - it's no longer needed.

#### 7.2 Consider archiving the backend

The backend still holds historical data. Options:
- **Export history**: Pull all historical snapshots before it dies (if needed for charts)
- **Keep as backup**: Leave it running as a fallback
- **Let it die**: If the new system works, just let the backend run out of cycles

#### 7.3 Update DNS/links

If you have custom domains or links pointing to the old backend, update them to point to the frontend canister which now serves all data.

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

## Decisions Made

| Question | Decision |
|----------|----------|
| Historical data structure | Store 168 hourly snapshots (7 days) in array format |
| SNS canisters | Full SNS root querying via `get_sns_canisters_summary` |
| Failed queries | Keep last known value, never lose data |
| Auto-deploy | Yes, deploy frontend after each successful collection |
| Git history growth | Accept ~4GB/year, simplicity over optimization |
| Monitoring | Use default GitHub Actions failure notifications |

## Potential Issues

1. **GitHub Actions timeout**: Collection takes ~5-10 minutes for 3,000+ canisters.
   GitHub Actions has a 6-hour timeout, so this is fine.

2. **Rate limiting**: The IC might rate-limit rapid queries.
   Mitigated by batching (50 concurrent) with delays between batches.

3. **Stale data on failures**: If collection fails completely, frontend shows old data.
   The "last updated" timestamp makes this visible to users.

4. **Large initial load**: ~3-5MB snapshots.json on first load.
   Consider adding loading states or pagination if this becomes an issue.

## Checklist

### Phase 1: Setup
- [ ] Create `cyclescan-deploy` dfx identity
- [ ] Get principal: `dfx identity get-principal`
- [ ] Add as frontend controller: `dfx canister update-settings ...`
- [ ] Export and base64 encode PEM
- [ ] Add `DFX_IDENTITY_PEM_B64` secret to GitHub

### Phase 2: Collection Script
- [ ] Create `scripts/package.json`
- [ ] Create `scripts/collect_snapshots.mjs`
- [ ] Test locally: `cd scripts && npm install && node collect_snapshots.mjs`
- [ ] Verify `data/live/snapshots.json` created

### Phase 3: GitHub Workflow
- [ ] Create `.github/workflows/collect-snapshots.yml`
- [ ] Commit and push
- [ ] Run workflow manually to test
- [ ] Verify data committed and frontend deployed

### Phase 4: Frontend
- [ ] Create `src/cyclescan_frontend/src/lib/data.ts`
- [ ] Update main page to use new data loading
- [ ] Update `dfx.json` to include data directory
- [ ] Test locally with `npm run preview`

### Phase 5: Go Live
- [ ] Do initial manual deploy
- [ ] Wait for first automated run
- [ ] Verify hourly updates working
- [ ] Monitor for a few days

### Phase 6: Cleanup
- [ ] Stop backend timer (optional)
- [ ] Update any external links
- [ ] Celebrate 98% cycles savings
