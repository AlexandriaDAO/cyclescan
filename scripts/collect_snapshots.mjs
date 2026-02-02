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

// FunnAI API getLatestDailyMetric interface (query method)
const funnaiApiIdl = ({ IDL }) => {
  const TierBreakdown = IDL.Record({
    low: IDL.Nat,
    custom: IDL.Nat,
    high: IDL.Nat,
    very_high: IDL.Nat,
    medium: IDL.Nat,
  });
  const MainerTotals = IDL.Record({
    created: IDL.Nat,
    active: IDL.Nat,
    total_cycles: IDL.Nat,
    paused: IDL.Nat,
  });
  const DailyBurnRate = IDL.Record({
    usd: IDL.Float64,
    cycles: IDL.Nat,
  });
  const CycleAmount = IDL.Record({
    usd: IDL.Float64,
    cycles: IDL.Nat,
  });
  const TotalCycles = IDL.Record({
    all: CycleAmount,
    mainers: CycleAmount,
    protocol: CycleAmount,
  });
  const SystemMetrics = IDL.Record({
    funnai_index: IDL.Float64,
    daily_burn_rate: DailyBurnRate,
    total_cycles: IDL.Opt(TotalCycles),
  });
  const DailyMetric = IDL.Record({
    mainers: IDL.Record({
      totals: MainerTotals,
      breakdown_by_tier: IDL.Record({
        active: TierBreakdown,
        paused: TierBreakdown,
      }),
    }),
    system_metrics: SystemMetrics,
    metadata: IDL.Record({
      updated_at: IDL.Text,
      date: IDL.Text,
      created_at: IDL.Text,
    }),
    derived_metrics: IDL.Record({
      avg_cycles_per_mainer: IDL.Float64,
      paused_percentage: IDL.Float64,
      tier_distribution: IDL.Record({
        low: IDL.Float64,
        custom: IDL.Float64,
        high: IDL.Float64,
        very_high: IDL.Float64,
        medium: IDL.Float64,
      }),
      burn_rate_per_active_mainer: IDL.Float64,
      active_percentage: IDL.Float64,
    }),
  });
  const ApiError = IDL.Variant({
    FailedOperation: IDL.Null,
    InvalidId: IDL.Null,
    ZeroAddress: IDL.Null,
    Unauthorized: IDL.Null,
    StatusCode: IDL.Nat16,
    Other: IDL.Text,
    InsuffientCycles: IDL.Nat,
  });
  const DailyMetricResult = IDL.Variant({
    Ok: DailyMetric,
    Err: ApiError,
  });
  return IDL.Service({
    getLatestDailyMetric: IDL.Func(
      [],
      [DailyMetricResult],
      ['query']
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
  const canistersPath = join(DATA_DIR, 'archive', 'canisters_backup.json');
  const data = JSON.parse(readFileSync(canistersPath, 'utf-8'));
  // Return all canisters - the valid flag is used for frontend display filtering only
  return data;
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

// FunnAI API canister ID
const FUNNAI_API_CANISTER = 'bgm6p-5aaaa-aaaaf-qbzda-cai';
const FUNNAI_AGGREGATE_ID = 'funnai-aggregate';
const TRILLION = 1_000_000_000_000n;

async function queryFunnaiApi(agent) {
  try {
    console.log(`\nQuerying FunnAI API (${FUNNAI_API_CANISTER})...`);
    const actor = Actor.createActor(funnaiApiIdl, {
      agent,
      canisterId: FUNNAI_API_CANISTER,
    });

    const result = await withTimeout(
      actor.getLatestDailyMetric(),
      30000,
      'Timeout querying FunnAI API'
    );

    if ('Ok' in result) {
      const metrics = result.Ok;

      // Get mAIners cycles from new total_cycles field in system_metrics
      // Falls back to mainers.totals.total_cycles if total_cycles not present
      let totalCyclesTrillion;
      if (metrics.system_metrics.total_cycles?.[0]?.mainers) {
        totalCyclesTrillion = BigInt(metrics.system_metrics.total_cycles[0].mainers.cycles);
      } else {
        totalCyclesTrillion = BigInt(metrics.mainers.totals.total_cycles);
      }
      const totalCycles = totalCyclesTrillion * TRILLION;

      // daily_burn_rate.cycles is in trillions
      const dailyBurnTrillion = BigInt(metrics.system_metrics.daily_burn_rate.cycles);

      // Convert daily burn to per-hour for consistency with our rate calculations
      const dailyBurnCycles = dailyBurnTrillion * TRILLION;
      const hourlyBurnCycles = dailyBurnCycles / 24n;

      console.log(`  FunnAI mAIners: ${metrics.mainers.totals.active} active / ${metrics.mainers.totals.created} total`);
      console.log(`  mAIner cycles: ${totalCyclesTrillion}T`);
      console.log(`  Daily burn: ${dailyBurnTrillion}T cycles/day`);

      return {
        totalCycles: totalCycles.toString(),
        hourlyBurnRate: hourlyBurnCycles.toString(),  // Store hourly rate for consistency
        activeMainers: Number(metrics.mainers.totals.active),
        totalMainers: Number(metrics.mainers.totals.created),
      };
    } else {
      console.error(`  FunnAI API returned error:`, result.Err);
    }
  } catch (e) {
    console.error(`  Failed to query FunnAI API: ${e.message}`);
  }
  return null;
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

  // -------------------------------------------------------------------------
  // Query FunnAI API for aggregate mAIner data
  // Synthesize declining balance from burn rate so sparklines work
  // -------------------------------------------------------------------------
  const burnRates = {};  // Store API-provided burn rates (per hour)
  const funnaiData = await queryFunnaiApi(agent);
  if (funnaiData) {
    const hourlyBurn = BigInt(funnaiData.hourlyBurnRate);
    const previousBalance = lastKnownBalances[FUNNAI_AGGREGATE_ID]
      ? BigInt(lastKnownBalances[FUNNAI_AGGREGATE_ID])
      : null;

    let syntheticBalance;
    if (previousBalance !== null) {
      // Subtract hourly burn from previous balance to create declining trend
      syntheticBalance = previousBalance - hourlyBurn;
      // Don't let it go negative
      if (syntheticBalance < 0n) syntheticBalance = 0n;
      console.log(`  FunnAI synthetic balance: ${previousBalance} - ${hourlyBurn} = ${syntheticBalance}`);
    } else {
      // First run: use API's total_cycles as starting point
      syntheticBalance = BigInt(funnaiData.totalCycles);
      console.log(`  FunnAI initial balance from API: ${syntheticBalance}`);
    }

    finalBalances[FUNNAI_AGGREGATE_ID] = syntheticBalance.toString();
    burnRates[FUNNAI_AGGREGATE_ID] = funnaiData.hourlyBurnRate;
    console.log(`  FunnAI burn rate: ${funnaiData.hourlyBurnRate}/hr`);
  } else if (lastKnownBalances[FUNNAI_AGGREGATE_ID]) {
    // API failed - use last known values and continue declining
    const lastBurnRates = existing.snapshots[0]?.burn_rates || {};
    const lastBurnRate = lastBurnRates[FUNNAI_AGGREGATE_ID];

    if (lastBurnRate) {
      // Continue declining with last known burn rate
      const previousBalance = BigInt(lastKnownBalances[FUNNAI_AGGREGATE_ID]);
      const hourlyBurn = BigInt(lastBurnRate);
      let syntheticBalance = previousBalance - hourlyBurn;
      if (syntheticBalance < 0n) syntheticBalance = 0n;

      finalBalances[FUNNAI_AGGREGATE_ID] = syntheticBalance.toString();
      burnRates[FUNNAI_AGGREGATE_ID] = lastBurnRate;
      console.log(`  FunnAI (API failed): ${previousBalance} - ${hourlyBurn} = ${syntheticBalance}`);
    } else {
      // No burn rate available, keep last balance
      finalBalances[FUNNAI_AGGREGATE_ID] = lastKnownBalances[FUNNAI_AGGREGATE_ID];
      console.log(`  FunnAI aggregate: using last known value (no burn rate)`);
    }
  }

  console.log(`\nFinal balances: ${Object.keys(finalBalances).length} canisters`);
  console.log(`  - Fresh queries: ${currentBalances.size}`);
  console.log(`  - From last known: ${Object.keys(finalBalances).length - currentBalances.size - (funnaiData ? 1 : 0)}`);

  // Create new snapshot
  const newSnapshot = {
    timestamp: Date.now(),
    balances: finalBalances,
    ...(Object.keys(burnRates).length > 0 && { burn_rates: burnRates }),
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
