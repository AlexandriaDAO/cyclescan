// src/cyclescan_frontend/src/lib/regression.ts
// Simplified burn rate calculation - one universal algorithm

export interface BurnRateData {
  rate: bigint;              // cycles per hour
  dataPoints: number;        // total snapshots used
  actualHours: number;       // actual time span of data
  topUpCount: number;        // number of top-ups detected
  totalTopUps: bigint;       // sum of all top-ups
  hasInferredData: boolean;  // true if any intervals were inferred
}

export interface IntervalData {
  startTime: number;
  endTime: number;
  duration: number;
  actualBurn: number;        // positive = burning, 0 if top-up interval
  inferredBurn: number;      // estimated burn during top-up intervals
  topUpAmount: number;       // positive if top-up occurred, 0 otherwise
  isTopUp: boolean;
}

/**
 * Analyze all intervals between snapshots.
 * Returns detailed interval data for both calculations and chart display.
 */
export function analyzeIntervals(
  points: Array<{ t: number; v: number }>
): IntervalData[] {
  if (points.length < 2) return [];

  const sorted = [...points].sort((a, b) => a.t - b.t);
  const intervals: IntervalData[] = [];

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
 * Safely convert a balance string to number for calculations.
 * Note: Very large balances (>9 quadrillion) may lose precision,
 * but this is acceptable for rate calculations.
 */
function safeBalanceToNumber(balanceStr: string): number {
  return Number(BigInt(balanceStr));
}

const MS_PER_HOUR = 3600000;

/**
 * Calculate burn rate from snapshots within a time window.
 * Uses one universal algorithm: average burn rate from actual burn intervals,
 * with inferred values for top-up intervals.
 */
export function calculateBurnRate(
  snapshots: Array<{ timestamp: number; balances: Record<string, string> }>,
  windowMs: number,
  now: number,
  canisterId: string
): BurnRateData | null {
  // Collect points for this canister within the window
  const cutoff = now - windowMs;
  const points: Array<{ t: number; v: number }> = [];

  for (const snapshot of snapshots) {
    if (snapshot.timestamp < cutoff) continue;
    const balanceStr = snapshot.balances[canisterId];
    if (!balanceStr) continue;
    points.push({
      t: snapshot.timestamp,
      v: safeBalanceToNumber(balanceStr),
    });
  }

  if (points.length < 2) return null;

  const intervals = analyzeIntervals(points);
  if (intervals.length === 0) return null;

  // Sum up actual and inferred burns
  const totalActualBurn = intervals.reduce((s, i) => s + i.actualBurn, 0);
  const totalInferredBurn = intervals.reduce((s, i) => s + i.inferredBurn, 0);
  const totalBurn = totalActualBurn + totalInferredBurn;

  // Total duration is the full time span
  const sorted = [...points].sort((a, b) => a.t - b.t);
  const fullDuration = sorted[sorted.length - 1].t - sorted[0].t;

  if (fullDuration <= 0) return null;

  // Burn rate = total burn / full duration
  const burnPerMs = totalBurn / fullDuration;
  const burnPerHour = burnPerMs * MS_PER_HOUR;

  // Count top-ups
  const topUpIntervals = intervals.filter(i => i.isTopUp);
  const totalTopUps = topUpIntervals.reduce((s, i) => s + i.topUpAmount, 0);

  return {
    rate: BigInt(Math.round(Math.max(0, burnPerHour))),
    dataPoints: points.length,
    actualHours: fullDuration / MS_PER_HOUR,
    topUpCount: topUpIntervals.length,
    totalTopUps: BigInt(Math.round(totalTopUps)),
    hasInferredData: topUpIntervals.length > 0,
  };
}

/**
 * Calculate burn rates for all three time windows.
 */
export function calculateAllBurnRates(
  snapshots: Array<{ timestamp: number; balances: Record<string, string> }>,
  canisterId: string
): {
  recent: BurnRateData | null;
  shortTerm: BurnRateData | null;
  longTerm: BurnRateData | null;
} {
  const now = snapshots[0]?.timestamp ?? Date.now();

  const HOUR = 3600000;
  const DAY = 24 * HOUR;
  const WEEK = 7 * DAY;

  return {
    recent: calculateBurnRate(snapshots, 2 * HOUR, now, canisterId),
    shortTerm: calculateBurnRate(snapshots, 36 * HOUR, now, canisterId),
    longTerm: calculateBurnRate(snapshots, WEEK, now, canisterId),
  };
}

/**
 * Aggregate project rates from canister rates.
 */
export interface ProjectRateData {
  rate: bigint;              // Sum of all canister rates
  totalDataPoints: number;   // Sum of all data points
  canistersWithData: number; // How many canisters contributed
  topUpsDetected: number;    // How many canisters had top-ups
  hasInferredData: boolean;  // True if any canister has inferred data
}

export function aggregateProjectRate(
  canisterRates: (BurnRateData | null)[]
): ProjectRateData | null {
  const validRates = canisterRates.filter((r): r is BurnRateData => r !== null);
  if (validRates.length === 0) return null;

  return {
    rate: validRates.reduce((sum, r) => sum + r.rate, 0n),
    totalDataPoints: validRates.reduce((sum, r) => sum + r.dataPoints, 0),
    canistersWithData: validRates.length,
    topUpsDetected: validRates.filter(r => r.topUpCount > 0).length,
    hasInferredData: validRates.some(r => r.hasInferredData),
  };
}

/**
 * Get interval data for chart display.
 * Returns data suitable for showing actual vs inferred burns.
 */
export function getIntervalsForChart(
  snapshots: Array<{ timestamp: number; balances: Record<string, string> }>,
  windowMs: number,
  now: number,
  canisterId: string
): IntervalData[] {
  const cutoff = now - windowMs;
  const points: Array<{ t: number; v: number }> = [];

  for (const snapshot of snapshots) {
    if (snapshot.timestamp < cutoff) continue;
    const balanceStr = snapshot.balances[canisterId];
    if (!balanceStr) continue;
    points.push({
      t: snapshot.timestamp,
      v: safeBalanceToNumber(balanceStr),
    });
  }

  return analyzeIntervals(points);
}
