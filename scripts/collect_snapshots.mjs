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
const BATCH_SIZE = 100; // Concurrent requests per batch (increased for speed)

// ============================================================================
// IDL Definitions
// ============================================================================

// Blackhole canister_status interface (works for any blackhole controller)
// Note: canister_status is NOT a query method - it requires an update call
// We use a minimal IDL that only extracts the cycles field we need
const blackholeIdl = ({ IDL }) => {
  return IDL.Service({
    canister_status: IDL.Func(
      [IDL.Record({ canister_id: IDL.Principal })],
      [IDL.Record({
        cycles: IDL.Nat,
        // Other fields exist but we don't need them; IDL record decoding is lenient
      })],
      []  // Update call, not query
    ),
  });
};

// SNS Root get_sns_canisters_summary interface
// Note: This is an update call, not a query
const snsRootIdl = ({ IDL }) => {
  const CanisterStatusResult = IDL.Record({
    cycles: IDL.Nat,
  });
  const CanisterSummary = IDL.Record({
    canister_id: IDL.Opt(IDL.Principal),
    status: IDL.Opt(CanisterStatusResult),
  });
  const GetSnsCanistersSummaryRequest = IDL.Record({
    update_canister_list: IDL.Opt(IDL.Bool),
  });
  return IDL.Service({
    get_sns_canisters_summary: IDL.Func(
      [GetSnsCanistersSummaryRequest],
      [IDL.Record({
        root: IDL.Opt(CanisterSummary),
        governance: IDL.Opt(CanisterSummary),
        ledger: IDL.Opt(CanisterSummary),
        swap: IDL.Opt(CanisterSummary),
        index: IDL.Opt(CanisterSummary),
        archives: IDL.Vec(CanisterSummary),
        dapps: IDL.Vec(CanisterSummary),
      })],
      []  // Update call, not query
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

// Timeout wrapper for promises
function withTimeout(promise, ms, errorMsg) {
  return Promise.race([
    promise,
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error(errorMsg)), ms)
    )
  ]);
}

async function queryBlackhole(agent, proxyId, canisterId) {
  try {
    const actor = Actor.createActor(blackholeIdl, {
      agent,
      canisterId: proxyId, // Use the canister's specific proxy (blackhole controller)
    });
    const result = await withTimeout(
      actor.canister_status({ canister_id: Principal.fromText(canisterId) }),
      30000,  // 30 second timeout per query
      `Timeout querying ${canisterId}`
    );
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
    const result = await withTimeout(
      actor.get_sns_canisters_summary({ update_canister_list: [] }),
      60000,  // 60 second timeout for SNS queries (they return more data)
      `Timeout querying SNS root ${snsRootId}`
    );

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

  for (let i = 0; i < snsRoots.length; i++) {
    const snsRootId = snsRoots[i];
    process.stdout.write(`  SNS ${i + 1}/${snsRoots.length} (${snsRootId.slice(0, 5)}...)...`);
    const snsBalances = await querySnsRoot(agent, snsRootId);
    for (const [id, balance] of snsBalances) {
      results.set(id, balance);
    }
    console.log(` ${snsBalances.size} canisters`);
    // Small delay between SNS queries
    await new Promise(r => setTimeout(r, 100));
  }
  console.log(`  Total SNS canisters: ${results.size}`);

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

  // Create agent (anonymous - works for update calls too)
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
