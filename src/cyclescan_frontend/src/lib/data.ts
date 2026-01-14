// src/cyclescan_frontend/src/lib/data.ts
// Data loading utilities - fetches directly from GitHub

import {
  calculateBurnRate,
  aggregateProjectRate,
  getIntervalsForChart,
  type BurnRateData,
  type ProjectRateData,
  type IntervalData,
} from './regression';

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

export type { BurnRateData, ProjectRateData, IntervalData };

export interface CanisterEntry {
  canister_id: string;
  project: string[] | null;
  balance: bigint;
  valid: boolean;
  recent_rate: BurnRateData | null;
  short_term_rate: BurnRateData | null;
  long_term_rate: BurnRateData | null;
}

export interface ProjectEntry {
  project: string;
  canister_count: bigint;
  total_balance: bigint;
  website: string[] | null;
  recent_rate: ProjectRateData | null;
  short_term_rate: ProjectRateData | null;
  long_term_rate: ProjectRateData | null;
}

export interface Stats {
  canister_count: bigint;
  snapshot_count: number;
  last_updated: Date | null;
}

let cachedData: {
  snapshots: SnapshotsData;
  canisters: CanisterRegistry[];
  projects: ProjectMeta[];
  entries: CanisterEntry[];
  projectEntries: ProjectEntry[];
  stats: Stats;
} | null = null;

const HOUR_MS = 60 * 60 * 1000;
const DAY_MS = 24 * HOUR_MS;

export async function loadData(): Promise<{
  entries: CanisterEntry[];
  projectEntries: ProjectEntry[];
  stats: Stats;
  rawSnapshots: Array<{ timestamp: number }>;
}> {
  if (cachedData) {
    return {
      entries: cachedData.entries,
      projectEntries: cachedData.projectEntries,
      stats: cachedData.stats,
      rawSnapshots: cachedData.snapshots.snapshots.map(s => ({ timestamp: s.timestamp })),
    };
  }

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

  const projectMetaMap = new Map<string, ProjectMeta>();
  for (const p of projectsMeta) {
    projectMetaMap.set(p.name, p);
  }

  const currentSnapshot = snapshots[0] || { balances: {}, timestamp: Date.now() };
  const now = currentSnapshot.timestamp;

  const entries: CanisterEntry[] = [];
  const projectAggregates = new Map<string, {
    count: bigint;
    balance: bigint;
    recentRates: (BurnRateData | null)[];
    shortTermRates: (BurnRateData | null)[];
    longTermRates: (BurnRateData | null)[];
  }>();

  for (const canister of canistersRegistry) {
    const balanceStr = currentSnapshot.balances[canister.canister_id];
    if (!balanceStr) continue;

    const balance = BigInt(balanceStr);
    const recentRate = calculateBurnRate(snapshots, 2 * HOUR_MS, now, canister.canister_id);
    const shortTermRate = calculateBurnRate(snapshots, 36 * HOUR_MS, now, canister.canister_id);
    const longTermRate = calculateBurnRate(snapshots, 7 * DAY_MS, now, canister.canister_id);

    entries.push({
      canister_id: canister.canister_id,
      project: canister.project,
      balance,
      valid: canister.valid,
      recent_rate: recentRate,
      short_term_rate: shortTermRate,
      long_term_rate: longTermRate,
    });

    const projectName = canister.project?.[0];
    if (projectName) {
      let agg = projectAggregates.get(projectName);
      if (!agg) {
        agg = { count: 0n, balance: 0n, recentRates: [], shortTermRates: [], longTermRates: [] };
        projectAggregates.set(projectName, agg);
      }
      agg.count += 1n;
      agg.balance += balance;
      agg.recentRates.push(recentRate);
      agg.shortTermRates.push(shortTermRate);
      agg.longTermRates.push(longTermRate);
    }
  }

  const projectEntries: ProjectEntry[] = [];
  for (const [projectName, agg] of projectAggregates) {
    const meta = projectMetaMap.get(projectName);
    projectEntries.push({
      project: projectName,
      canister_count: agg.count,
      total_balance: agg.balance,
      website: meta?.website || null,
      recent_rate: aggregateProjectRate(agg.recentRates),
      short_term_rate: aggregateProjectRate(agg.shortTermRates),
      long_term_rate: aggregateProjectRate(agg.longTermRates),
    });
  }

  projectEntries.sort((a, b) => {
    const aRate = a.short_term_rate?.rate ?? -1n;
    const bRate = b.short_term_rate?.rate ?? -1n;
    if (bRate > aRate) return 1;
    if (bRate < aRate) return -1;
    return 0;
  });

  const stats: Stats = {
    canister_count: BigInt(entries.length),
    snapshot_count: snapshots.length,
    last_updated: snapshots[0] ? new Date(snapshots[0].timestamp) : null,
  };

  cachedData = { snapshots: snapshotsData, canisters: canistersRegistry, projects: projectsMeta, entries, projectEntries, stats };

  return { entries, projectEntries, stats, rawSnapshots: snapshots.map(s => ({ timestamp: s.timestamp })) };
}

export async function getProjectCanisters(projectName: string): Promise<CanisterEntry[]> {
  if (!cachedData) await loadData();
  return cachedData?.entries.filter(e => e.project?.[0] === projectName) ?? [];
}

export interface CanisterDetail {
  project: string[] | null;
  current_balance: bigint;
  recent_rate: BurnRateData | null;
  short_term_rate: BurnRateData | null;
  long_term_rate: BurnRateData | null;
  snapshots: Array<{ timestamp: bigint; cycles: bigint }>;
}

export async function getCanisterDetail(canisterId: string): Promise<CanisterDetail | null> {
  if (!cachedData) await loadData();
  if (!cachedData) return null;

  const { snapshots } = cachedData.snapshots;
  const canisterRegistry = cachedData.canisters.find(c => c.canister_id === canisterId);
  if (!canisterRegistry) return null;

  const currentBalanceStr = snapshots[0]?.balances[canisterId];
  if (!currentBalanceStr) return null;

  const now = snapshots[0].timestamp;
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
    current_balance: BigInt(currentBalanceStr),
    recent_rate: calculateBurnRate(snapshots, 2 * HOUR_MS, now, canisterId),
    short_term_rate: calculateBurnRate(snapshots, 36 * HOUR_MS, now, canisterId),
    long_term_rate: calculateBurnRate(snapshots, 7 * DAY_MS, now, canisterId),
    snapshots: snapshotHistory,
  };
}

export function getChartIntervals(canisterId: string, windowMs: number): IntervalData[] {
  if (!cachedData) return [];
  const snapshots = cachedData.snapshots.snapshots;
  const now = snapshots[0]?.timestamp ?? Date.now();
  return getIntervalsForChart(snapshots, windowMs, now, canisterId);
}

export function getProjectSparklineIntervals(projectName: string, windowMs: number): IntervalData[] {
  if (!cachedData) return [];

  const projectCanisters = cachedData.canisters.filter(c => c.project?.[0] === projectName);
  if (projectCanisters.length === 0) return [];

  const snapshots = cachedData.snapshots.snapshots;
  const now = snapshots[0]?.timestamp ?? Date.now();

  const canisterIntervals = projectCanisters.map(c =>
    getIntervalsForChart(snapshots, windowMs, now, c.canister_id)
  );

  // Aggregate intervals from all canisters
  const allTimestamps = new Set<number>();
  for (const intervals of canisterIntervals) {
    for (const interval of intervals) {
      allTimestamps.add(interval.startTime);
      allTimestamps.add(interval.endTime);
    }
  }

  const sortedTimestamps = [...allTimestamps].sort((a, b) => a - b);
  if (sortedTimestamps.length < 2) return [];

  const aggregated: IntervalData[] = [];
  for (let i = 1; i < sortedTimestamps.length; i++) {
    const startTime = sortedTimestamps[i - 1];
    const endTime = sortedTimestamps[i];
    let totalActualBurn = 0, totalInferredBurn = 0, totalTopUpAmount = 0, hasTopUp = false;

    for (const intervals of canisterIntervals) {
      const match = intervals.find(int => int.startTime <= startTime && int.endTime >= endTime);
      if (match) {
        const ratio = match.duration > 0 ? (endTime - startTime) / match.duration : 0;
        totalActualBurn += match.actualBurn * ratio;
        totalInferredBurn += match.inferredBurn * ratio;
        totalTopUpAmount += match.topUpAmount * ratio;
        if (match.isTopUp) hasTopUp = true;
      }
    }

    aggregated.push({
      startTime,
      endTime,
      duration: endTime - startTime,
      actualBurn: totalActualBurn,
      inferredBurn: totalInferredBurn,
      topUpAmount: totalTopUpAmount,
      isTopUp: hasTopUp,
    });
  }

  return aggregated;
}
