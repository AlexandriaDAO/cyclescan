// src/cyclescan_frontend/src/lib/regression.ts
// Linear regression utilities for calculating burn rates from irregular snapshot data

export interface RegressionResult {
  slope: number;           // cycles per millisecond (raw)
  intercept: number;       // theoretical balance at t=0
  r2: number;              // goodness of fit (0-1)
  n: number;               // number of points used
  timeSpanMs: number;      // actual time span of data used
}

export interface BurnRateData {
  rate: bigint;             // cycles per hour (positive = burning)
  confidence: number;       // R² (0-1)
  dataPoints: number;       // snapshots used
  actualHours: number;      // actual time span of data
  topUpActivity: 'none' | 'single' | 'frequent';
  netBurnRate: bigint | null;  // burn rate excluding top-ups (null if uncalculable)
  totalTopUps: bigint;      // sum of all detected top-ups in window
}

export interface TopUpAnalysis {
  activity: 'none' | 'single' | 'frequent';
  topUps: Array<{ index: number; amount: number }>;
  totalAmount: number;
}

/**
 * Safely convert a balance string to number for regression math.
 * Number.MAX_SAFE_INTEGER ≈ 9×10^15, largest expected balance ≈ 100T = 10^14
 * This gives us 90x safety margin.
 */
function safeBalanceToNumber(balanceStr: string): number {
  const balance = BigInt(balanceStr);

  // Safety check
  if (balance > 9_000_000_000_000_000n) {
    console.warn(`Balance ${balanceStr} exceeds safe integer range, precision may be lost`);
  }

  return Number(balance);
}

/**
 * Compute linear regression for a set of time-value points.
 * Returns null if insufficient data.
 */
export function linearRegression(
  points: Array<{ t: number; v: number }>
): RegressionResult | null {
  const n = points.length;
  if (n < 2) return null;

  let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0, sumY2 = 0;
  let minT = Infinity, maxT = -Infinity;

  for (const { t, v } of points) {
    sumX += t;
    sumY += v;
    sumXY += t * v;
    sumX2 += t * t;
    sumY2 += v * v;
    minT = Math.min(minT, t);
    maxT = Math.max(maxT, t);
  }

  const denom = n * sumX2 - sumX * sumX;
  if (denom === 0) return null;

  const slope = (n * sumXY - sumX * sumY) / denom;
  const intercept = (sumY - slope * sumX) / n;

  // Calculate R² (coefficient of determination)
  const yMean = sumY / n;
  const ssTotal = sumY2 - n * yMean * yMean;
  const ssResidual = sumY2 - intercept * sumY - slope * sumXY;
  const r2 = ssTotal === 0 ? 1 : Math.max(0, 1 - (ssResidual / ssTotal));

  return {
    slope,
    intercept,
    r2,
    n,
    timeSpanMs: maxT - minT,
  };
}

/**
 * Detect top-up patterns in the data.
 * Top-ups are positive jumps in balance between consecutive snapshots.
 */
export function analyzeTopUps(points: Array<{ t: number; v: number }>): TopUpAnalysis {
  if (points.length < 2) {
    return { activity: 'none', topUps: [], totalAmount: 0 };
  }

  const sorted = [...points].sort((a, b) => a.t - b.t);
  const topUps: Array<{ index: number; amount: number }> = [];

  // Threshold for detecting a top-up: 100B cycles
  // This is small enough to catch most top-ups but large enough to filter noise
  const THRESHOLD = 100_000_000_000;

  for (let i = 1; i < sorted.length; i++) {
    const jump = sorted[i].v - sorted[i - 1].v;
    if (jump > THRESHOLD) {
      topUps.push({ index: i, amount: jump });
    }
  }

  // Classify the pattern
  let activity: 'none' | 'single' | 'frequent';
  if (topUps.length === 0) {
    activity = 'none';
  } else if (topUps.length === 1) {
    activity = 'single';
  } else {
    activity = 'frequent';  // 2+ top-ups in window
  }

  return {
    activity,
    topUps,
    totalAmount: topUps.reduce((sum, t) => sum + t.amount, 0),
  };
}

/**
 * Calculate burn rate from snapshots within a time window.
 * Uses linear regression and handles top-up patterns.
 *
 * @param snapshots - Array of {timestamp: ms, balances: Record<string, string>} objects
 * @param windowMs - How far back to look (e.g., 2 hours = 7200000)
 * @param now - Current timestamp in ms
 * @param canisterId - The canister to calculate for
 */
export function calculateBurnRateWithTopUps(
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

  const analysis = analyzeTopUps(points);
  const sorted = [...points].sort((a, b) => a.t - b.t);
  const MS_PER_HOUR = 3600000;

  switch (analysis.activity) {
    case 'none': {
      // Simple case: no top-ups, just run regression
      const result = linearRegression(points);
      if (!result) return null;

      // slope is cycles per millisecond
      // Negative slope = balance decreasing = burning cycles
      const burnPerMs = -result.slope;
      const burnPerHour = burnPerMs * MS_PER_HOUR;

      return {
        rate: BigInt(Math.round(Math.max(0, burnPerHour))),
        confidence: result.r2,
        dataPoints: result.n,
        actualHours: result.timeSpanMs / MS_PER_HOUR,
        topUpActivity: 'none',
        totalTopUps: 0n,
        netBurnRate: BigInt(Math.round(Math.max(0, burnPerHour))),
      };
    }

    case 'single': {
      // One top-up: segment after it for accurate burn rate
      const lastTopUp = analysis.topUps[analysis.topUps.length - 1];
      const postTopUpPoints = sorted.slice(lastTopUp.index);

      if (postTopUpPoints.length >= 2) {
        const result = linearRegression(postTopUpPoints);
        if (result) {
          const burnPerMs = -result.slope;
          const burnPerHour = burnPerMs * MS_PER_HOUR;

          return {
            rate: BigInt(Math.round(Math.max(0, burnPerHour))),
            confidence: result.r2,
            dataPoints: result.n,
            actualHours: result.timeSpanMs / MS_PER_HOUR,
            topUpActivity: 'single',
            totalTopUps: BigInt(Math.round(analysis.totalAmount)),
            netBurnRate: BigInt(Math.round(Math.max(0, burnPerHour))),
          };
        }
      }
      // Fall through if not enough post-top-up data
      // Use full regression but flag the top-up
      const result = linearRegression(points);
      if (!result) return null;

      const burnPerMs = -result.slope;
      const burnPerHour = burnPerMs * MS_PER_HOUR;

      return {
        rate: BigInt(Math.round(burnPerHour)),  // May be negative (net gaining)
        confidence: result.r2,
        dataPoints: result.n,
        actualHours: result.timeSpanMs / MS_PER_HOUR,
        topUpActivity: 'single',
        totalTopUps: BigInt(Math.round(analysis.totalAmount)),
        netBurnRate: null,  // Can't calculate reliable burn rate
      };
    }

    case 'frequent': {
      // Frequent top-ups: can't segment, report net rate AND estimated burn
      const result = linearRegression(points);
      if (!result) return null;

      // Net rate (includes top-ups) - may be negative (gaining)
      const burnPerMs = -result.slope;
      const netBurnPerHour = burnPerMs * MS_PER_HOUR;

      // Estimate "pure" burn by looking at intervals WITHOUT top-ups
      const burnIntervals: Array<{ burn: number; duration: number }> = [];
      for (let i = 1; i < sorted.length; i++) {
        const change = sorted[i].v - sorted[i - 1].v;
        if (change <= 0) {  // Only burning intervals (no top-up)
          const duration = sorted[i].t - sorted[i - 1].t;
          burnIntervals.push({ burn: -change, duration });
        }
      }

      let estimatedBurnRate: bigint | null = null;
      if (burnIntervals.length > 0) {
        const totalBurn = burnIntervals.reduce((s, i) => s + i.burn, 0);
        const totalDuration = burnIntervals.reduce((s, i) => s + i.duration, 0);
        if (totalDuration > 0) {
          estimatedBurnRate = BigInt(Math.round((totalBurn / totalDuration) * MS_PER_HOUR));
        }
      }

      return {
        rate: BigInt(Math.round(netBurnPerHour)),  // Net rate (may be negative)
        confidence: result.r2,
        dataPoints: result.n,
        actualHours: result.timeSpanMs / MS_PER_HOUR,
        topUpActivity: 'frequent',
        totalTopUps: BigInt(Math.round(analysis.totalAmount)),
        netBurnRate: estimatedBurnRate,  // Burn rate from non-top-up intervals
      };
    }
  }
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

  // Use slightly larger windows to ensure we capture enough data
  // For recent: use last 2 hours of data
  // For short-term: use last 36 hours of data
  // For long-term: use all available data (7 days)

  return {
    recent: calculateBurnRateWithTopUps(snapshots, 2 * HOUR, now, canisterId),
    shortTerm: calculateBurnRateWithTopUps(snapshots, 36 * HOUR, now, canisterId),
    longTerm: calculateBurnRateWithTopUps(snapshots, WEEK, now, canisterId),
  };
}

/**
 * Aggregate project rates from canister rates.
 * R² doesn't sum - use weighted average by data points.
 */
export interface ProjectRateData {
  rate: bigint;                          // Sum of all canister rates
  avgConfidence: number;                 // Weighted average R²
  totalDataPoints: number;               // Sum of all data points
  canistersWithData: number;             // How many canisters contributed
  canistersWithLowConfidence: number;    // How many had R² < 0.5
  topUpsDetected: number;                // How many canisters had top-ups
}

export function aggregateProjectRate(canisterRates: (BurnRateData | null)[]): ProjectRateData | null {
  const validRates = canisterRates.filter((r): r is BurnRateData => r !== null);
  if (validRates.length === 0) return null;

  const totalRate = validRates.reduce((sum, r) => sum + r.rate, 0n);
  const totalPoints = validRates.reduce((sum, r) => sum + r.dataPoints, 0);

  // Weighted average: Σ(R² × dataPoints) / Σ(dataPoints)
  const weightedR2Sum = validRates.reduce(
    (sum, r) => sum + r.confidence * r.dataPoints, 0
  );
  const avgConfidence = totalPoints > 0 ? weightedR2Sum / totalPoints : 0;

  return {
    rate: totalRate,
    avgConfidence,
    totalDataPoints: totalPoints,
    canistersWithData: validRates.length,
    canistersWithLowConfidence: validRates.filter(r => r.confidence < 0.5).length,
    topUpsDetected: validRates.filter(r => r.topUpActivity !== 'none').length,
  };
}
