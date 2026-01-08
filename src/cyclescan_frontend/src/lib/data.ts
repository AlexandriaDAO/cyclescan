// src/cyclescan_frontend/src/lib/data.ts
// Data loading utilities - fetches directly from GitHub (no redeployment needed)
// Uses linear regression for burn rate calculation (see UI_DATA_REALITY.md)

import {
  calculateBurnRateWithTopUps,
  aggregateProjectRate,
  linearRegression,
  type BurnRateData,
  type ProjectRateData,
} from './regression';

// GitHub raw content base URL
const GITHUB_RAW_BASE = 'https://raw.githubusercontent.com/AlexandriaDAO/cyclescan/master/data';

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

export interface ProjectMeta {
  name: string;
  website: string[] | null;
}

// Re-export BurnRateData for external use
export type { BurnRateData, ProjectRateData };

// Updated CanisterEntry using regression-based rates
export interface CanisterEntry {
  canister_id: string;
  project: string[] | null;
  balance: bigint;
  valid: boolean;
  // New: regression-based rates (all stored as cycles/hour)
  recent_rate: BurnRateData | null;      // ~2h window
  short_term_rate: BurnRateData | null;  // ~36h window
  long_term_rate: BurnRateData | null;   // ~7d window
  // Legacy fields for backwards compatibility during migration
  burn_1h: [bigint] | [];
  burn_24h: [bigint] | [];
  burn_7d: [bigint] | [];
}

// Updated ProjectEntry with aggregated rates
export interface ProjectEntry {
  project: string;
  canister_count: bigint;
  total_balance: bigint;
  website: string[] | null;
  // New: aggregated rates
  recent_rate: ProjectRateData | null;
  short_term_rate: ProjectRateData | null;
  long_term_rate: ProjectRateData | null;
  // Legacy fields for backwards compatibility
  total_burn_1h: [bigint] | [];
  total_burn_24h: [bigint] | [];
  total_burn_7d: [bigint] | [];
}

// Global stats with aggregated network burn rates
export interface Stats {
  canister_count: bigint;
  snapshot_count: number;
  last_updated: Date | null;
  // New: network-wide burn rates
  network_burn: {
    recent: bigint | null;
    shortTerm: bigint | null;
    longTerm: bigint | null;
    avgConfidence: number;
  };
}

// Time windows info for display
export interface TimeWindows {
  recent: { targetHours: number; actualHours: number; available: boolean };
  daily: { targetHours: number; actualHours: number; available: boolean };
  weekly: { targetHours: number; actualHours: number; available: boolean };
}

// Cache for loaded data
let cachedData: {
  snapshots: SnapshotsData;
  canisters: CanisterRegistry[];
  projects: ProjectMeta[];
  entries: CanisterEntry[];
  projectEntries: ProjectEntry[];
  stats: Stats;
  timeWindows: TimeWindows;
} | null = null;

// Time constants in milliseconds
const HOUR_MS = 60 * 60 * 1000;
const DAY_MS = 24 * HOUR_MS;

// Legacy: tolerance-based snapshot finding (kept for backwards compat)
const TIME_TOLERANCE_1H = 0.75;
const TIME_TOLERANCE = 0.5;

function findSnapshotNearTime(
  snapshots: Snapshot[],
  targetTimeMs: number,
  toleranceMs: number
): Snapshot | null {
  if (snapshots.length === 0) return null;

  let bestSnapshot: Snapshot | null = null;
  let bestDelta = Infinity;

  for (const snapshot of snapshots) {
    const delta = Math.abs(snapshot.timestamp - targetTimeMs);
    if (delta < bestDelta) {
      bestDelta = delta;
      bestSnapshot = snapshot;
    }
  }

  if (bestSnapshot && bestDelta <= toleranceMs) {
    return bestSnapshot;
  }
  return null;
}

function calculateBurn(current: bigint, previousStr: string | undefined): bigint | null {
  if (!previousStr) return null;
  const previous = BigInt(previousStr);
  return previous > current ? previous - current : 0n;
}

export async function loadData(): Promise<{
  entries: CanisterEntry[];
  projectEntries: ProjectEntry[];
  stats: Stats;
  timeWindows: TimeWindows;
}> {
  // Return cached data if available
  if (cachedData) {
    return {
      entries: cachedData.entries,
      projectEntries: cachedData.projectEntries,
      stats: cachedData.stats,
      timeWindows: cachedData.timeWindows,
    };
  }

  // Fetch data directly from GitHub (cache-bust with timestamp for snapshots)
  const cacheBust = `?t=${Math.floor(Date.now() / 60000)}`;
  const [snapshotsRes, canistersRes, projectsRes] = await Promise.all([
    fetch(`${GITHUB_RAW_BASE}/live/snapshots.json${cacheBust}`),
    fetch(`${GITHUB_RAW_BASE}/archive/canisters_backup.json`),
    fetch(`${GITHUB_RAW_BASE}/archive/projects_backup.json`),
  ]);

  const snapshotsData: SnapshotsData = await snapshotsRes.json();
  const canistersRegistry: CanisterRegistry[] = await canistersRes.json();
  const projectsMeta: ProjectMeta[] = await projectsRes.json();

  const { snapshots } = snapshotsData;

  // Build project metadata lookup
  const projectMetaMap = new Map<string, ProjectMeta>();
  for (const p of projectsMeta) {
    projectMetaMap.set(p.name, p);
  }

  const currentSnapshot = snapshots[0] || { balances: {}, timestamp: Date.now() };
  const now = currentSnapshot.timestamp;

  // Legacy: find snapshots near target times
  const recentSnap = findSnapshotNearTime(snapshots, now - HOUR_MS, HOUR_MS * TIME_TOLERANCE_1H);
  const dailySnap = findSnapshotNearTime(snapshots, now - DAY_MS, DAY_MS * TIME_TOLERANCE);
  const weeklySnap = findSnapshotNearTime(snapshots, now - 7 * DAY_MS, 7 * DAY_MS * TIME_TOLERANCE);

  // Build time windows for UI display
  const timeWindows: TimeWindows = {
    recent: {
      targetHours: 2,
      actualHours: recentSnap ? (now - recentSnap.timestamp) / HOUR_MS : 0,
      available: snapshots.length >= 2,
    },
    daily: {
      targetHours: 36,
      actualHours: dailySnap ? (now - dailySnap.timestamp) / HOUR_MS : 0,
      available: snapshots.length >= 2,
    },
    weekly: {
      targetHours: 168,
      actualHours: weeklySnap ? (now - weeklySnap.timestamp) / HOUR_MS : 0,
      available: snapshots.length >= 2,
    },
  };

  // Legacy snapshot balances for backwards compat
  const snapshot1h = recentSnap?.balances || {};
  const snapshot24h = dailySnap?.balances || {};
  const snapshot7d = weeklySnap?.balances || {};

  // Build canister entries with regression-based rates
  const entries: CanisterEntry[] = [];
  const projectAggregates = new Map<string, {
    count: bigint;
    balance: bigint;
    recentRates: (BurnRateData | null)[];
    shortTermRates: (BurnRateData | null)[];
    longTermRates: (BurnRateData | null)[];
    // Legacy
    burn1h: bigint;
    burn24h: bigint;
    burn7d: bigint;
    has1h: boolean;
    has24h: boolean;
    has7d: boolean;
  }>();

  for (const canister of canistersRegistry) {
    const balanceStr = currentSnapshot.balances[canister.canister_id];
    if (!balanceStr) continue;

    const balance = BigInt(balanceStr);

    // Calculate burn rates using linear regression
    const recentRate = calculateBurnRateWithTopUps(snapshots, 2 * HOUR_MS, now, canister.canister_id);
    const shortTermRate = calculateBurnRateWithTopUps(snapshots, 36 * HOUR_MS, now, canister.canister_id);
    const longTermRate = calculateBurnRateWithTopUps(snapshots, 7 * DAY_MS, now, canister.canister_id);

    // Legacy burn calculations
    const burn1h = calculateBurn(balance, snapshot1h[canister.canister_id]);
    const burn24h = calculateBurn(balance, snapshot24h[canister.canister_id]);
    const burn7d = calculateBurn(balance, snapshot7d[canister.canister_id]);

    const entry: CanisterEntry = {
      canister_id: canister.canister_id,
      project: canister.project,
      balance,
      valid: canister.valid,
      // New regression-based rates
      recent_rate: recentRate,
      short_term_rate: shortTermRate,
      long_term_rate: longTermRate,
      // Legacy fields
      burn_1h: burn1h !== null ? [burn1h] : [],
      burn_24h: burn24h !== null ? [burn24h] : [],
      burn_7d: burn7d !== null ? [burn7d] : [],
    };
    entries.push(entry);

    // Aggregate by project
    const projectName = canister.project?.[0];
    if (projectName) {
      let agg = projectAggregates.get(projectName);
      if (!agg) {
        agg = {
          count: 0n,
          balance: 0n,
          recentRates: [],
          shortTermRates: [],
          longTermRates: [],
          burn1h: 0n,
          burn24h: 0n,
          burn7d: 0n,
          has1h: false,
          has24h: false,
          has7d: false,
        };
        projectAggregates.set(projectName, agg);
      }
      agg.count += 1n;
      agg.balance += balance;
      agg.recentRates.push(recentRate);
      agg.shortTermRates.push(shortTermRate);
      agg.longTermRates.push(longTermRate);

      if (burn1h !== null) {
        agg.burn1h += burn1h;
        agg.has1h = true;
      }
      if (burn24h !== null) {
        agg.burn24h += burn24h;
        agg.has24h = true;
      }
      if (burn7d !== null) {
        agg.burn7d += burn7d;
        agg.has7d = true;
      }
    }
  }

  // Build project entries with aggregated rates
  const projectEntries: ProjectEntry[] = [];
  for (const [projectName, agg] of projectAggregates) {
    const meta = projectMetaMap.get(projectName);

    projectEntries.push({
      project: projectName,
      canister_count: agg.count,
      total_balance: agg.balance,
      website: meta?.website || null,
      // Aggregated regression rates
      recent_rate: aggregateProjectRate(agg.recentRates),
      short_term_rate: aggregateProjectRate(agg.shortTermRates),
      long_term_rate: aggregateProjectRate(agg.longTermRates),
      // Legacy
      total_burn_1h: agg.has1h ? [agg.burn1h] : [],
      total_burn_24h: agg.has24h ? [agg.burn24h] : [],
      total_burn_7d: agg.has7d ? [agg.burn7d] : [],
    });
  }

  // Sort project entries by short-term rate descending (nulls at bottom)
  projectEntries.sort((a, b) => {
    const aRate = a.short_term_rate?.rate ?? -1n;
    const bRate = b.short_term_rate?.rate ?? -1n;
    if (bRate > aRate) return 1;
    if (bRate < aRate) return -1;
    return 0;
  });

  // Calculate network-wide stats
  const allRecentRates = entries.map(e => e.recent_rate).filter((r): r is BurnRateData => r !== null);
  const allShortTermRates = entries.map(e => e.short_term_rate).filter((r): r is BurnRateData => r !== null);
  const allLongTermRates = entries.map(e => e.long_term_rate).filter((r): r is BurnRateData => r !== null);

  const networkRecent = allRecentRates.length > 0
    ? allRecentRates.reduce((sum, r) => sum + r.rate, 0n)
    : null;
  const networkShortTerm = allShortTermRates.length > 0
    ? allShortTermRates.reduce((sum, r) => sum + r.rate, 0n)
    : null;
  const networkLongTerm = allLongTermRates.length > 0
    ? allLongTermRates.reduce((sum, r) => sum + r.rate, 0n)
    : null;

  // Weighted average confidence
  const totalPoints = allShortTermRates.reduce((sum, r) => sum + r.dataPoints, 0);
  const avgConfidence = totalPoints > 0
    ? allShortTermRates.reduce((sum, r) => sum + r.confidence * r.dataPoints, 0) / totalPoints
    : 0;

  const stats: Stats = {
    canister_count: BigInt(entries.length),
    snapshot_count: snapshots.length,
    last_updated: snapshots[0] ? new Date(snapshots[0].timestamp) : null,
    network_burn: {
      recent: networkRecent,
      shortTerm: networkShortTerm,
      longTerm: networkLongTerm,
      avgConfidence,
    },
  };

  // Cache the data
  cachedData = {
    snapshots: snapshotsData,
    canisters: canistersRegistry,
    projects: projectsMeta,
    entries,
    projectEntries,
    stats,
    timeWindows,
  };

  return { entries, projectEntries, stats, timeWindows };
}

export async function getProjectCanisters(projectName: string): Promise<CanisterEntry[]> {
  if (!cachedData) {
    await loadData();
  }

  if (!cachedData) {
    return [];
  }

  return cachedData.entries.filter(e => e.project?.[0] === projectName);
}

// Burn measurement with honest time delta (for modal display)
export interface BurnMeasurement {
  amount: bigint;
  actualHours: number;
  available: boolean;
}

// Canister detail for the modal
export interface CanisterDetail {
  project: string[] | null;
  current_balance: bigint;
  // New: regression-based burn rates
  recent_rate: BurnRateData | null;
  short_term_rate: BurnRateData | null;
  long_term_rate: BurnRateData | null;
  // Legacy burn measurements
  recent_burn: BurnMeasurement;
  daily_burn: BurnMeasurement;
  weekly_burn: BurnMeasurement;
  burn_1h: [bigint] | [];
  burn_24h: [bigint] | [];
  burn_7d: [bigint] | [];
  burn_30d: [bigint] | [];
  snapshots: Array<{ timestamp: bigint; cycles: bigint }>;
}

export async function getCanisterDetail(canisterId: string): Promise<CanisterDetail | null> {
  if (!cachedData) {
    await loadData();
  }

  if (!cachedData) {
    return null;
  }

  const { snapshots } = cachedData.snapshots;
  const canisterRegistry = cachedData.canisters.find(c => c.canister_id === canisterId);

  if (!canisterRegistry) {
    return null;
  }

  const currentBalanceStr = snapshots[0]?.balances[canisterId];
  if (!currentBalanceStr) {
    return null;
  }

  const currentBalance = BigInt(currentBalanceStr);
  const now = snapshots[0].timestamp;

  // Calculate regression-based rates
  const recentRate = calculateBurnRateWithTopUps(snapshots, 2 * HOUR_MS, now, canisterId);
  const shortTermRate = calculateBurnRateWithTopUps(snapshots, 36 * HOUR_MS, now, canisterId);
  const longTermRate = calculateBurnRateWithTopUps(snapshots, 7 * DAY_MS, now, canisterId);

  // Legacy: find snapshots near target times
  const snap1h = findSnapshotNearTime(snapshots, now - HOUR_MS, HOUR_MS * TIME_TOLERANCE_1H);
  const snap24h = findSnapshotNearTime(snapshots, now - DAY_MS, DAY_MS * TIME_TOLERANCE);
  const snap7d = findSnapshotNearTime(snapshots, now - 7 * DAY_MS, 7 * DAY_MS * TIME_TOLERANCE);

  function computeBurnMeasurement(snap: Snapshot | null): BurnMeasurement {
    if (!snap) {
      return { amount: 0n, actualHours: 0, available: false };
    }
    const balanceStr = snap.balances[canisterId];
    if (!balanceStr) {
      return { amount: 0n, actualHours: 0, available: false };
    }
    const previousBalance = BigInt(balanceStr);
    const burnAmount = previousBalance - currentBalance;
    const actualHours = (now - snap.timestamp) / HOUR_MS;
    return { amount: burnAmount, actualHours, available: true };
  }

  const recent_burn = computeBurnMeasurement(snap1h);
  const daily_burn = computeBurnMeasurement(snap24h);
  const weekly_burn = computeBurnMeasurement(snap7d);

  const burn1h = recent_burn.available && recent_burn.amount > 0n ? recent_burn.amount : null;
  const burn24h = daily_burn.available && daily_burn.amount > 0n ? daily_burn.amount : null;
  const burn7d = weekly_burn.available && weekly_burn.amount > 0n ? weekly_burn.amount : null;

  // Build snapshots array for chart
  const snapshotHistory: Array<{ timestamp: bigint; cycles: bigint }> = [];
  for (const snapshot of snapshots) {
    const cyclesStr = snapshot.balances[canisterId];
    if (cyclesStr) {
      snapshotHistory.push({
        timestamp: BigInt(snapshot.timestamp) * 1_000_000n,
        cycles: BigInt(cyclesStr),
      });
    }
  }

  return {
    project: canisterRegistry.project,
    current_balance: currentBalance,
    // New regression-based rates
    recent_rate: recentRate,
    short_term_rate: shortTermRate,
    long_term_rate: longTermRate,
    // Legacy fields
    recent_burn,
    daily_burn,
    weekly_burn,
    burn_1h: burn1h !== null ? [burn1h] : [],
    burn_24h: burn24h !== null ? [burn24h] : [],
    burn_7d: burn7d !== null ? [burn7d] : [],
    burn_30d: [],
    snapshots: snapshotHistory,
  };
}

// Get raw snapshots for chart regression line
export function getRawSnapshots(): Snapshot[] {
  return cachedData?.snapshots.snapshots ?? [];
}

export function clearCache() {
  cachedData = null;
}
