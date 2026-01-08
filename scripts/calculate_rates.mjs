// scripts/calculate_rates.mjs
//
// Reads snapshots.json and calculates burn rates for all canisters.
// Outputs rates.json with pre-computed values for the frontend.
//
// Algorithm ported from src/cyclescan_frontend/src/lib/regression.ts
// Uses two-pass approach to handle top-ups correctly.

import { readFileSync, writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DATA_DIR = join(__dirname, '..', 'data');

// Time windows for burn rate calculations (matching frontend)
const HOUR = 60 * 60 * 1000;
const WINDOWS = {
  recent: 2 * HOUR,       // Last 2 hours
  short_term: 36 * HOUR,  // Last 36 hours (matches frontend)
  long_term: 168 * HOUR,  // Last 7 days
};

// ============================================================================
// Data Loading
// ============================================================================

function loadSnapshots() {
  const path = join(DATA_DIR, 'live', 'snapshots.json');
  const data = JSON.parse(readFileSync(path, 'utf-8'));
  return data.snapshots; // Array of { timestamp, balances }
}

function loadCanisters() {
  const path = join(DATA_DIR, 'archive', 'canisters_backup.json');
  return JSON.parse(readFileSync(path, 'utf-8'));
}

function loadProjects() {
  const path = join(DATA_DIR, 'archive', 'projects_backup.json');
  return JSON.parse(readFileSync(path, 'utf-8'));
}

// ============================================================================
// Burn Rate Calculation (ported from regression.ts)
// ============================================================================

/**
 * Analyze all intervals between snapshots.
 * Two-pass algorithm:
 * 1. First pass: identify burn vs top-up intervals
 * 2. Calculate average burn rate from actual burn intervals
 * 3. Second pass: fill in inferred burn for top-up intervals
 */
function analyzeIntervals(points) {
  if (points.length < 2) return [];

  const sorted = [...points].sort((a, b) => a.t - b.t);
  const intervals = [];

  // First pass: identify actual burn intervals vs top-up intervals
  for (let i = 1; i < sorted.length; i++) {
    const change = sorted[i].v - sorted[i - 1].v;
    const duration = sorted[i].t - sorted[i - 1].t;

    if (duration <= 0) continue;

    intervals.push({
      startTime: sorted[i - 1].t,
      endTime: sorted[i].t,
      duration,
      actualBurn: change <= 0 ? -change : 0,  // Burning = balance decreased
      inferredBurn: 0,  // Will be filled in second pass
      topUpAmount: change > 0 ? change : 0,   // Top-up = balance increased
      isTopUp: change > 0,
    });
  }

  // Calculate average burn rate from actual burn intervals
  const burnIntervals = intervals.filter(i => !i.isTopUp);
  const totalActualBurn = burnIntervals.reduce((s, i) => s + i.actualBurn, 0);
  const totalBurnDuration = burnIntervals.reduce((s, i) => s + i.duration, 0);
  const avgBurnPerMs = totalBurnDuration > 0 ? totalActualBurn / totalBurnDuration : 0;

  // Second pass: fill in inferred burn for top-up intervals
  for (const interval of intervals) {
    if (interval.isTopUp) {
      interval.inferredBurn = avgBurnPerMs * interval.duration;
    }
  }

  return intervals;
}

/**
 * Extract balance points for a canister within a time window.
 */
function getBalancePoints(snapshots, canisterId, windowMs, now) {
  const cutoff = now - windowMs;
  const points = [];

  for (const snapshot of snapshots) {
    if (snapshot.timestamp < cutoff) continue;

    const balanceStr = snapshot.balances[canisterId];
    if (balanceStr !== undefined) {
      points.push({
        t: snapshot.timestamp,
        v: Number(BigInt(balanceStr)), // Convert to number for calculations
      });
    }
  }

  return points;
}

/**
 * Calculate burn rate from balance history points.
 */
function calculateBurnRate(points) {
  if (points.length < 2) {
    return { rate: 0n, topUpCount: 0, hasInferredData: false, dataPoints: 0 };
  }

  const intervals = analyzeIntervals(points);
  if (intervals.length === 0) {
    return { rate: 0n, topUpCount: 0, hasInferredData: false, dataPoints: points.length };
  }

  // Sum up actual and inferred burns
  const totalActualBurn = intervals.reduce((s, i) => s + i.actualBurn, 0);
  const totalInferredBurn = intervals.reduce((s, i) => s + i.inferredBurn, 0);
  const totalBurn = totalActualBurn + totalInferredBurn;

  // Total duration is the full time span
  const sorted = [...points].sort((a, b) => a.t - b.t);
  const fullDuration = sorted[sorted.length - 1].t - sorted[0].t;

  if (fullDuration <= 0) {
    return { rate: 0n, topUpCount: 0, hasInferredData: false, dataPoints: points.length };
  }

  // Burn rate = total burn / full duration, converted to per-hour
  const burnPerMs = totalBurn / fullDuration;
  const burnPerHour = burnPerMs * HOUR;

  // Count top-ups
  const topUpIntervals = intervals.filter(i => i.isTopUp);
  const totalTopUps = topUpIntervals.reduce((s, i) => s + i.topUpAmount, 0);

  return {
    rate: BigInt(Math.round(Math.max(0, burnPerHour))),
    topUpCount: topUpIntervals.length,
    totalTopUps: BigInt(Math.round(totalTopUps)),
    hasInferredData: topUpIntervals.length > 0,
    dataPoints: points.length,
  };
}

/**
 * Calculate runway (days until zero) from balance and burn rate.
 */
function calculateRunway(balance, ratePerHour) {
  if (ratePerHour <= 0n) return null; // Infinite runway or gaining cycles

  const hoursRemaining = Number(balance) / Number(ratePerHour);
  const daysRemaining = hoursRemaining / 24;

  return Math.round(daysRemaining * 10) / 10; // One decimal place
}

// ============================================================================
// Main Processing
// ============================================================================

function processCanister(snapshots, canisterId, now) {
  // Get current balance
  const currentBalance = snapshots[0]?.balances[canisterId];
  if (currentBalance === undefined) {
    return null;
  }

  const balance = BigInt(currentBalance);

  // Calculate rates for each time window
  const rates = {};

  for (const [windowName, windowMs] of Object.entries(WINDOWS)) {
    const points = getBalancePoints(snapshots, canisterId, windowMs, now);
    rates[windowName] = calculateBurnRate(points);
  }

  // Use long_term rate for runway calculation (most stable)
  const primaryRate = rates.long_term.rate > 0n
    ? rates.long_term.rate
    : rates.short_term.rate;

  const runway = calculateRunway(balance, primaryRate);

  return {
    balance: currentBalance, // Keep as string for JSON
    rates: {
      recent: {
        rate: rates.recent.rate.toString(),
        hasInferredData: rates.recent.hasInferredData,
        dataPoints: rates.recent.dataPoints,
      },
      short_term: {
        rate: rates.short_term.rate.toString(),
        hasInferredData: rates.short_term.hasInferredData,
        dataPoints: rates.short_term.dataPoints,
      },
      long_term: {
        rate: rates.long_term.rate.toString(),
        hasInferredData: rates.long_term.hasInferredData,
        dataPoints: rates.long_term.dataPoints,
      },
    },
    runway,
    topUpCount: rates.long_term.topUpCount,
  };
}

function processProject(canisterRates, canisterInfoList) {
  // Sum balances
  let totalBalance = 0n;
  let canisterCount = 0;

  for (const info of canisterInfoList) {
    const rates = canisterRates[info.canister_id];
    if (!rates) continue;

    totalBalance += BigInt(rates.balance);
    canisterCount++;
  }

  if (canisterCount === 0) {
    return null;
  }

  // For project rates, sum the individual canister rates
  let recentRate = 0n;
  let shortTermRate = 0n;
  let longTermRate = 0n;
  let hasInferredData = false;
  let totalTopUps = 0;
  let totalDataPoints = { recent: 0, short_term: 0, long_term: 0 };

  for (const info of canisterInfoList) {
    const rates = canisterRates[info.canister_id];
    if (!rates) continue;

    recentRate += BigInt(rates.rates.recent.rate);
    shortTermRate += BigInt(rates.rates.short_term.rate);
    longTermRate += BigInt(rates.rates.long_term.rate);
    hasInferredData = hasInferredData || rates.rates.long_term.hasInferredData;
    totalTopUps += rates.topUpCount;
    totalDataPoints.recent += rates.rates.recent.dataPoints;
    totalDataPoints.short_term += rates.rates.short_term.dataPoints;
    totalDataPoints.long_term += rates.rates.long_term.dataPoints;
  }

  const primaryRate = longTermRate > 0n ? longTermRate : shortTermRate;
  const runway = calculateRunway(totalBalance, primaryRate);

  return {
    canister_count: canisterCount,
    total_balance: totalBalance.toString(),
    rates: {
      recent: { rate: recentRate.toString(), hasInferredData, dataPoints: totalDataPoints.recent },
      short_term: { rate: shortTermRate.toString(), hasInferredData, dataPoints: totalDataPoints.short_term },
      long_term: { rate: longTermRate.toString(), hasInferredData, dataPoints: totalDataPoints.long_term },
    },
    runway,
    topUpCount: totalTopUps,
  };
}

async function main() {
  console.log('='.repeat(60));
  console.log('CycleScan Rate Calculation');
  console.log(`Time: ${new Date().toISOString()}`);
  console.log('='.repeat(60));

  // Load data
  console.log('\nLoading data...');
  const snapshots = loadSnapshots();
  const canisters = loadCanisters();
  const projects = loadProjects();

  console.log(`  Snapshots: ${snapshots.length}`);
  console.log(`  Canisters: ${canisters.length}`);
  console.log(`  Projects: ${projects.length}`);

  if (snapshots.length === 0) {
    console.error('No snapshots found!');
    process.exit(1);
  }

  const now = snapshots[0]?.timestamp || Date.now();
  console.log(`  Latest snapshot: ${new Date(now).toISOString()}`);

  // Process all canisters
  console.log('\nProcessing canisters...');
  const canisterRates = {};
  let processed = 0;
  let skipped = 0;

  for (const canister of canisters) {
    if (canister.valid === false) {
      skipped++;
      continue;
    }

    const rates = processCanister(snapshots, canister.canister_id, now);
    if (rates) {
      canisterRates[canister.canister_id] = {
        ...rates,
        project: canister.project?.[0] || 'Unknown',
      };
      processed++;
    } else {
      skipped++;
    }
  }

  console.log(`  Processed: ${processed}`);
  console.log(`  Skipped: ${skipped}`);

  // Group canisters by project
  console.log('\nProcessing projects...');
  const projectMap = new Map();

  for (const canister of canisters) {
    if (canister.valid === false) continue;
    const projectName = canister.project?.[0] || 'Unknown';
    if (!projectMap.has(projectName)) {
      projectMap.set(projectName, []);
    }
    projectMap.get(projectName).push(canister);
  }

  // Calculate project-level stats
  const projectRates = {};

  for (const [projectName, canisterList] of projectMap) {
    const stats = processProject(canisterRates, canisterList);
    if (stats) {
      // Find project metadata (website)
      const projectMeta = projects.find(p => p.name === projectName);
      projectRates[projectName] = {
        ...stats,
        website: projectMeta?.website?.[0] || null,
      };
    }
  }

  console.log(`  Projects with data: ${Object.keys(projectRates).length}`);

  // Build output
  const output = {
    generated_at: Date.now(),
    snapshot_count: snapshots.length,
    latest_snapshot: snapshots[0]?.timestamp,
    oldest_snapshot: snapshots[snapshots.length - 1]?.timestamp,
    windows: {
      recent: '2h',
      short_term: '36h',
      long_term: '7d',
    },
    canisters: canisterRates,
    projects: projectRates,
  };

  // Write output
  mkdirSync(join(DATA_DIR, 'live'), { recursive: true });
  const outputPath = join(DATA_DIR, 'live', 'rates.json');
  writeFileSync(outputPath, JSON.stringify(output));

  const fileSizeKB = Math.round(JSON.stringify(output).length / 1024);
  console.log(`\nWrote ${outputPath}`);
  console.log(`  Size: ~${fileSizeKB}KB`);
  console.log(`  Canisters: ${Object.keys(canisterRates).length}`);
  console.log(`  Projects: ${Object.keys(projectRates).length}`);
  console.log('='.repeat(60));
}

main().catch(e => {
  console.error('Rate calculation failed:', e);
  process.exit(1);
});
