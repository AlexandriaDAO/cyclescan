// src/cyclescan_frontend/src/lib/data.ts
// Data loading utilities - fetches directly from GitHub (no redeployment needed)

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

export interface CanisterEntry {
  canister_id: string;
  project: string[] | null;
  balance: bigint;
  burn_1h: [bigint] | [];
  burn_24h: [bigint] | [];
  burn_7d: [bigint] | [];
  valid: boolean;
}

export interface ProjectEntry {
  project: string;
  canister_count: bigint;
  total_balance: bigint;
  total_burn_1h: [bigint] | [];
  total_burn_24h: [bigint] | [];
  total_burn_7d: [bigint] | [];
  website: string[] | null;
}

export interface Stats {
  canister_count: bigint;
  snapshot_count: number;
  last_updated: Date | null;
}

// Cache for loaded data
let cachedData: {
  snapshots: SnapshotsData;
  canisters: CanisterRegistry[];
  projects: ProjectMeta[];
  entries: CanisterEntry[];
  projectEntries: ProjectEntry[];
  stats: Stats;
} | null = null;

// Time constants in milliseconds
const HOUR_MS = 60 * 60 * 1000;
const DAY_MS = 24 * HOUR_MS;

// Tolerance: how far from target time a snapshot can be (as fraction of period)
// 1h needs higher tolerance since gaps in hourly collection are more impactful
const TIME_TOLERANCE_1H = 0.75; // 45 mins for 1h (handles missed runs)
const TIME_TOLERANCE = 0.5;     // 50% for 24h and 7d

/**
 * Find a snapshot closest to the target time, within tolerance.
 * Returns null if no snapshot is within acceptable range.
 */
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

  // Only return if within tolerance
  if (bestSnapshot && bestDelta <= toleranceMs) {
    return bestSnapshot;
  }
  return null;
}

function calculateBurn(current: bigint, previousStr: string | undefined): bigint | null {
  if (!previousStr) return null;
  const previous = BigInt(previousStr);
  // Burn = previous - current (positive if cycles decreased)
  return previous > current ? previous - current : 0n;
}

export async function loadData(): Promise<{
  entries: CanisterEntry[];
  projectEntries: ProjectEntry[];
  stats: Stats;
}> {
  // Return cached data if available
  if (cachedData) {
    return {
      entries: cachedData.entries,
      projectEntries: cachedData.projectEntries,
      stats: cachedData.stats,
    };
  }

  // Fetch data directly from GitHub (cache-bust with timestamp for snapshots)
  const cacheBust = `?t=${Math.floor(Date.now() / 60000)}`; // Changes every minute
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

  // Get current snapshot and calculate target times for burn comparisons
  const currentSnapshot = snapshots[0] || { balances: {}, timestamp: Date.now() };
  const now = currentSnapshot.timestamp;

  // Find snapshots near target times (using actual timestamps, not indices)
  const snapshot1h = findSnapshotNearTime(snapshots, now - HOUR_MS, HOUR_MS * TIME_TOLERANCE_1H)?.balances || {};
  const snapshot24h = findSnapshotNearTime(snapshots, now - DAY_MS, DAY_MS * TIME_TOLERANCE)?.balances || {};
  const snapshot7d = findSnapshotNearTime(snapshots, now - 7 * DAY_MS, 7 * DAY_MS * TIME_TOLERANCE)?.balances || {};

  // Build canister entries
  const entries: CanisterEntry[] = [];
  const projectAggregates = new Map<string, {
    count: bigint;
    balance: bigint;
    burn1h: bigint;
    burn24h: bigint;
    burn7d: bigint;
    has1h: boolean;
    has24h: boolean;
    has7d: boolean;
  }>();

  for (const canister of canistersRegistry) {
    const balanceStr = currentSnapshot.balances[canister.canister_id];
    if (!balanceStr) continue; // Skip canisters not in current snapshot

    const balance = BigInt(balanceStr);
    const burn1h = calculateBurn(balance, snapshot1h[canister.canister_id]);
    const burn24h = calculateBurn(balance, snapshot24h[canister.canister_id]);
    const burn7d = calculateBurn(balance, snapshot7d[canister.canister_id]);

    const entry: CanisterEntry = {
      canister_id: canister.canister_id,
      project: canister.project,
      balance,
      burn_1h: burn1h !== null ? [burn1h] : [],
      burn_24h: burn24h !== null ? [burn24h] : [],
      burn_7d: burn7d !== null ? [burn7d] : [],
      valid: canister.valid,
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

  // Build project entries
  const projectEntries: ProjectEntry[] = [];
  for (const [projectName, agg] of projectAggregates) {
    const meta = projectMetaMap.get(projectName);
    projectEntries.push({
      project: projectName,
      canister_count: agg.count,
      total_balance: agg.balance,
      total_burn_1h: agg.has1h ? [agg.burn1h] : [],
      total_burn_24h: agg.has24h ? [agg.burn24h] : [],
      total_burn_7d: agg.has7d ? [agg.burn7d] : [],
      website: meta?.website || null,
    });
  }

  // Sort project entries by 24h burn descending
  projectEntries.sort((a, b) => {
    const aVal = a.total_burn_24h[0] ?? -1n;
    const bVal = b.total_burn_24h[0] ?? -1n;
    if (bVal > aVal) return 1;
    if (bVal < aVal) return -1;
    return 0;
  });

  const stats: Stats = {
    canister_count: BigInt(entries.length),
    snapshot_count: snapshots.length,
    last_updated: snapshots[0] ? new Date(snapshots[0].timestamp) : null,
  };

  // Cache the data
  cachedData = {
    snapshots: snapshotsData,
    canisters: canistersRegistry,
    projects: projectsMeta,
    entries,
    projectEntries,
    stats,
  };

  return { entries, projectEntries, stats };
}

export async function getProjectCanisters(projectName: string): Promise<CanisterEntry[]> {
  // Ensure data is loaded
  if (!cachedData) {
    await loadData();
  }

  if (!cachedData) {
    return [];
  }

  // Filter entries by project name
  return cachedData.entries.filter(e => e.project?.[0] === projectName);
}

// Canister detail for the modal (matches backend.get_canister response)
export interface CanisterDetail {
  project: string[] | null;
  current_balance: bigint;
  burn_1h: [bigint] | [];
  burn_24h: [bigint] | [];
  burn_7d: [bigint] | [];
  burn_30d: [bigint] | [];  // Will be empty since we only have 7 days of data
  snapshots: Array<{ timestamp: bigint; cycles: bigint }>;
}

export async function getCanisterDetail(canisterId: string): Promise<CanisterDetail | null> {
  // Ensure data is loaded
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

  // Get current balance
  const currentBalanceStr = snapshots[0]?.balances[canisterId];
  if (!currentBalanceStr) {
    return null;
  }

  const currentBalance = BigInt(currentBalanceStr);
  const now = snapshots[0].timestamp;

  // Calculate burn rates using time-based snapshot lookup
  const snap1h = findSnapshotNearTime(snapshots, now - HOUR_MS, HOUR_MS * TIME_TOLERANCE_1H);
  const snap24h = findSnapshotNearTime(snapshots, now - DAY_MS, DAY_MS * TIME_TOLERANCE);
  const snap7d = findSnapshotNearTime(snapshots, now - 7 * DAY_MS, 7 * DAY_MS * TIME_TOLERANCE);

  const balance1h = snap1h?.balances[canisterId];
  const balance24h = snap24h?.balances[canisterId];
  const balance7d = snap7d?.balances[canisterId];

  const burn1h = balance1h ? BigInt(balance1h) - currentBalance : null;
  const burn24h = balance24h ? BigInt(balance24h) - currentBalance : null;
  const burn7d = balance7d ? BigInt(balance7d) - currentBalance : null;

  // Build snapshots array for chart (convert to nanoseconds as expected by modal)
  const snapshotHistory: Array<{ timestamp: bigint; cycles: bigint }> = [];
  for (const snapshot of snapshots) {
    const cyclesStr = snapshot.balances[canisterId];
    if (cyclesStr) {
      snapshotHistory.push({
        // Convert milliseconds to nanoseconds (modal expects nanoseconds)
        timestamp: BigInt(snapshot.timestamp) * 1_000_000n,
        cycles: BigInt(cyclesStr),
      });
    }
  }

  return {
    project: canisterRegistry.project,
    current_balance: currentBalance,
    burn_1h: burn1h !== null && burn1h > 0n ? [burn1h] : [],
    burn_24h: burn24h !== null && burn24h > 0n ? [burn24h] : [],
    burn_7d: burn7d !== null && burn7d > 0n ? [burn7d] : [],
    burn_30d: [],  // We don't have 30 days of data in static approach
    snapshots: snapshotHistory,
  };
}

// Clear the cache (useful for testing or forcing refresh)
export function clearCache() {
  cachedData = null;
}
