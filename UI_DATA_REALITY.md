# CycleScan UI: Designing for Data Reality

## The Problem

The current UI presents data as if we have precise, regular measurements:
- "1h Burn" implies exactly 1 hour ago
- "24h Burn" implies exactly 24 hours ago
- Chart intervals (1H, 4H, 12H, 1D) imply regular spacing

**Reality:** Our data collection has inherent variance that makes these labels misleading.

---

## Current Data Collection Reality

### Collection Mechanism
- GitHub Actions scheduled cron job at `:05` every hour
- Each run takes ~10-12 minutes to query ~2,900 canisters
- Data committed to repo, frontend fetches from GitHub raw URLs

### Actual Timing Variance

From recent data analysis:

| Metric | Value |
|--------|-------|
| Target interval | 60 mins |
| Actual min gap | 18 mins |
| Actual max gap | 174 mins (2.9 hours) |
| Actual average | 62 mins |

### Why Variance Exists

1. **GitHub Actions cron is "best effort"** - not guaranteed timing
2. **Query duration varies** - network latency, IC response times
3. **Occasional failures** - workflow may fail and retry next hour
4. **Git conflicts** - rebase/merge during concurrent pushes

### What This Means

When we say "1h Burn", we're actually showing:
> "The difference between current balance and the closest snapshot we have to 1 hour ago (±45 min tolerance)"

This could be comparing against a snapshot from 15 minutes ago or 1 hour 45 minutes ago.

---

## Current Implementation: Two-Point Comparison

### How It Works Now

```typescript
// Current approach in data.ts
const snapshot1h = findSnapshotNearTime(snapshots, now - HOUR_MS, HOUR_MS * 0.75);
const burn1h = currentBalance - snapshot1h.balance;
```

### The Flaws

1. **Dependent on finding "the right" snapshot** - if no snapshot exists near the target time, we get no data
2. **Uses only 2 data points** - ignores all other available information
3. **Sensitive to outliers** - one bad reading skews everything
4. **Misleading labels** - "1h Burn" suggests precision we don't have
5. **Arbitrary tolerance** - why 75%? Why not 50%? Why not 100%?

### Visual Example of the Problem

```
Actual snapshots:     •     •        •    •      •
Timeline:         |-----|-----|-----|-----|-----|
                  0h   1h    2h    3h    4h    5h ago

When user asks for "1h burn":
- Target: 1h ago
- Nearest snapshot: might be at 0.5h or 1.5h
- We compare current (0h) to whatever we found
- Result could represent 30 min or 90 min of burn
```

---

## The Solution: Linear Regression

### Core Insight

Instead of asking "what was the balance 1 hour ago?", ask:
> "What is the rate of change over the past hour's worth of data?"

### Mathematical Foundation

**Linear Regression** fits a line through all data points, minimizing squared error:

```
cycles = slope × time + intercept
```

Where:
- **slope** = the burn rate (cycles per millisecond)
- **intercept** = theoretical balance at time 0
- **R²** = how well the line fits (0 to 1)

### The Formula

For points `(x₁, y₁), (x₂, y₂), ..., (xₙ, yₙ)`:

```
slope = (n × Σxy - Σx × Σy) / (n × Σx² - (Σx)²)
```

In code:
```typescript
function linearRegression(points: Array<{t: number, v: number}>): {
  slope: number;      // cycles per millisecond
  intercept: number;  // theoretical y at x=0
  r2: number;         // goodness of fit (0-1)
} | null {
  const n = points.length;
  if (n < 2) return null;

  let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0, sumY2 = 0;

  for (const { t, v } of points) {
    sumX += t;
    sumY += v;
    sumXY += t * v;
    sumX2 += t * t;
    sumY2 += v * v;
  }

  const denom = n * sumX2 - sumX * sumX;
  if (denom === 0) return null;

  const slope = (n * sumXY - sumX * sumY) / denom;
  const intercept = (sumY - slope * sumX) / n;

  // Calculate R² (coefficient of determination)
  const yMean = sumY / n;
  const ssTotal = sumY2 - n * yMean * yMean;
  const ssResidual = sumY2 - intercept * sumY - slope * sumXY;
  const r2 = ssTotal === 0 ? 1 : 1 - (ssResidual / ssTotal);

  return { slope, intercept, r2 };
}
```

### Why This Is Better

| Aspect | Two-Point Comparison | Linear Regression |
|--------|---------------------|-------------------|
| Data used | 2 snapshots | ALL snapshots in window |
| Outlier sensitivity | High (one bad point ruins it) | Low (averaged out) |
| Missing data handling | Fails or uses wrong snapshot | Gracefully uses what exists |
| Statistical validity | None | Mathematically sound |
| Confidence measure | None | R² tells us fit quality |
| What it represents | "Difference over unknown time" | "Rate of change" |

### Visual Example

```
Actual snapshots:     •     •        •    •      •
                       \   /          \  /      /
                        \ /            \/      /
Best fit line:    ──────●──────────────●─────●──────
                        ↑
                     slope = burn rate

With regression:
- Uses ALL 5 points to determine the trend
- Outliers are smoothed out
- Slope directly gives us cycles/time
- R² tells us how confident we should be
```

---

## Performance Analysis

### Computational Complexity

Linear regression is O(n) - single pass through data:

```
For each point: 5 additions, 3 multiplications
Final calculation: ~10 operations
Total: 5n + 10 operations
```

### Benchmarks (Actual Test Results)

```
Configuration:
  - Canisters: 2,900
  - Snapshots: 168 (7 days)
  - Time windows: 3 (hourly, daily, weekly)
  - Total regressions: 8,700

Results:
  - Total computation time: 11.26ms
  - Per canister: 0.0039ms
  - Per regression: 0.0013ms
```

### Memory Usage

```
Per canister data structure:
  - 168 points × 16 bytes (timestamp + value) = 2.7 KB
  - 2,900 canisters × 2.7 KB = 7.8 MB

This is negligible for modern browsers.
```

### Conclusion

**Linear regression is trivially fast for our use case.** The 11ms computation time is imperceptible to users and happens once on data load.

---

## Implementation Plan

### Phase 1: Add Regression Utility Functions

Create new file: `src/cyclescan_frontend/src/lib/regression.ts`

```typescript
// src/cyclescan_frontend/src/lib/regression.ts

export interface RegressionResult {
  slope: number;           // cycles per millisecond (negative = burning)
  intercept: number;       // theoretical balance at t=0
  r2: number;              // goodness of fit (0-1)
  n: number;               // number of points used
  timeSpanMs: number;      // actual time span of data used
}

export interface BurnRate {
  ratePerHour: bigint;     // cycles burned per hour (positive = burning)
  ratePerDay: bigint;      // cycles burned per day
  confidence: number;      // R² value (0-1)
  dataPoints: number;      // how many snapshots contributed
  actualTimeSpan: number;  // actual time span in ms
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

  // Calculate R²
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
 * Calculate burn rate from snapshots within a time window.
 *
 * @param snapshots - Array of {timestamp: ms, balance: string} objects
 * @param windowMs - How far back to look (e.g., 1 hour = 3600000)
 * @param now - Current timestamp in ms
 * @param canisterId - The canister to calculate for
 */
export function calculateBurnRate(
  snapshots: Array<{ timestamp: number; balances: Record<string, string> }>,
  windowMs: number,
  now: number,
  canisterId: string
): BurnRate | null {
  // Collect points for this canister within the window
  const cutoff = now - windowMs;
  const points: Array<{ t: number; v: number }> = [];

  for (const snapshot of snapshots) {
    if (snapshot.timestamp < cutoff) continue;

    const balanceStr = snapshot.balances[canisterId];
    if (!balanceStr) continue;

    points.push({
      t: snapshot.timestamp,
      v: Number(BigInt(balanceStr)),  // Convert to number for regression math
    });
  }

  if (points.length < 2) return null;

  const result = linearRegression(points);
  if (!result) return null;

  // slope is cycles per millisecond
  // Negative slope = balance decreasing = burning cycles
  const burnPerMs = -result.slope;  // Make positive for "burn"
  const burnPerHour = burnPerMs * 3600000;
  const burnPerDay = burnPerHour * 24;

  return {
    ratePerHour: BigInt(Math.round(Math.max(0, burnPerHour))),
    ratePerDay: BigInt(Math.round(Math.max(0, burnPerDay))),
    confidence: result.r2,
    dataPoints: result.n,
    actualTimeSpan: result.timeSpanMs,
  };
}

/**
 * Calculate burn rates for all three time windows.
 */
export function calculateAllBurnRates(
  snapshots: Array<{ timestamp: number; balances: Record<string, string> }>,
  canisterId: string
): {
  hourly: BurnRate | null;
  daily: BurnRate | null;
  weekly: BurnRate | null;
} {
  const now = snapshots[0]?.timestamp ?? Date.now();

  const HOUR = 3600000;
  const DAY = 24 * HOUR;
  const WEEK = 7 * DAY;

  // Use slightly larger windows to ensure we capture enough data
  // For hourly: use last 2 hours of data
  // For daily: use last 36 hours of data
  // For weekly: use all available data

  return {
    hourly: calculateBurnRate(snapshots, 2 * HOUR, now, canisterId),
    daily: calculateBurnRate(snapshots, 36 * HOUR, now, canisterId),
    weekly: calculateBurnRate(snapshots, WEEK, now, canisterId),
  };
}
```

### Phase 2: Update Data Types

Modify `src/cyclescan_frontend/src/lib/data.ts`:

```typescript
// Updated interfaces - SEE "Implementation Specifications" SECTION FOR DEFINITIVE TYPES
// This is a summary; the specs section has the authoritative version.

export interface BurnRateData {
  rate: bigint;             // cycles per hour (positive = burning)
  confidence: number;       // R² (0-1)
  dataPoints: number;       // snapshots used
  actualHours: number;      // actual time span of data
  topUpActivity: 'none' | 'single' | 'frequent';
  netBurnRate: bigint | null;  // burn rate excluding top-ups
}

export interface CanisterEntry {
  canister_id: string;
  project: string[] | null;
  balance: bigint;
  valid: boolean;

  // Rates for different time windows (all stored as cycles/hour)
  recent_rate: BurnRateData | null;      // ~2h window
  short_term_rate: BurnRateData | null;  // ~36h window
  long_term_rate: BurnRateData | null;   // ~7d window
}

export interface ProjectEntry {
  project: string;
  canister_count: bigint;
  total_balance: bigint;
  website: string[] | null;

  // Aggregated rates - see Spec 3 for aggregation logic
  recent_rate: ProjectRateData | null;
  short_term_rate: ProjectRateData | null;
  long_term_rate: ProjectRateData | null;
}

export interface ProjectRateData {
  rate: bigint;                          // Sum of all canister rates
  avgConfidence: number;                 // Weighted average R²
  totalDataPoints: number;               // Sum of all data points
  canistersWithData: number;             // How many canisters contributed
  canistersWithLowConfidence: number;    // How many had R² < 0.5
}
```

### Phase 3: Update loadData() Function

```typescript
// In data.ts - updated loadData function
// SEE "Implementation Specifications" SECTION FOR DEFINITIVE LOGIC

import { calculateBurnRateWithTopUps } from './regression';

export async function loadData(): Promise<{
  entries: CanisterEntry[];
  projectEntries: ProjectEntry[];
  stats: GlobalStats;
}> {
  // ... fetch data as before ...

  const { snapshots } = snapshotsData;
  const now = snapshots[0]?.timestamp ?? Date.now();

  const HOUR = 3600000;
  const entries: CanisterEntry[] = [];

  for (const canister of canistersRegistry) {
    const balanceStr = snapshots[0]?.balances[canister.canister_id];
    if (!balanceStr) continue;

    const balance = BigInt(balanceStr);

    // Calculate burn rates using linear regression with top-up detection
    // Each returns BurnRateData (see Phase 2) or null
    const recent = calculateBurnRateWithTopUps(snapshots, 2 * HOUR, now, canister.canister_id);
    const shortTerm = calculateBurnRateWithTopUps(snapshots, 36 * HOUR, now, canister.canister_id);
    const longTerm = calculateBurnRateWithTopUps(snapshots, 7 * 24 * HOUR, now, canister.canister_id);

    entries.push({
      canister_id: canister.canister_id,
      project: canister.project,
      balance,
      valid: canister.valid,
      recent_rate: recent,
      short_term_rate: shortTerm,
      long_term_rate: longTerm,
    });
  }

  // Aggregate projects - see Spec 3 for R² aggregation logic
  const projectAggregates = aggregateProjects(entries, projectsMeta);

  // Calculate global stats - see Spec 6
  const stats = calculateGlobalStats(entries, snapshots);

  return { entries, projectEntries: projectAggregates, stats };
}
```

### Phase 4: Update UI Components

#### Table Headers

```svelte
<!-- Use the new naming from Spec 1 (Recent/Short-term/Long-term) -->
<th>
  Recent
  <span class="sublabel">(~2h)</span>
  <span class="info-icon" title="Burn rate from last ~2 hours of data">ⓘ</span>
</th>
<th>
  Short-term
  <span class="sublabel">(~24h)</span>
  <span class="info-icon" title="Burn rate from last ~24 hours of data">ⓘ</span>
</th>
<th>
  Long-term
  <span class="sublabel">(~7d)</span>
  <span class="info-icon" title="Burn rate from last ~7 days of data">ⓘ</span>
</th>
```

#### Table Cells

```svelte
<!-- All display uses /day suffix - see Spec 1 -->
<td class="burn">
  {#if entry.short_term_rate}
    <span
      class="rate-value"
      class:low-confidence={entry.short_term_rate.confidence < 0.5}
    >
      {formatRate(entry.short_term_rate.rate)}
    </span>
    {#if entry.short_term_rate.topUpActivity !== 'none'}
      <span class="topup-indicator">⚡</span>
    {/if}
  {:else}
    <span class="no-data">—</span>
  {/if}
</td>
```

#### Formatting Function

```typescript
// formatRate takes cycles/hour, displays as cycles/day
// This is the ONLY place the conversion happens - see Spec 1
function formatRate(ratePerHour: bigint): string {
  const perDay = ratePerHour * 24n;

  const TRILLION = 1_000_000_000_000n;
  const BILLION = 1_000_000_000n;

  // Handle negative rates (gaining)
  const isNegative = perDay < 0n;
  const absPerDay = isNegative ? -perDay : perDay;
  const prefix = isNegative ? '↑' : '';

  let formatted: string;
  if (absPerDay >= TRILLION) {
    formatted = (Number(absPerDay / BILLION) / 1000).toFixed(2) + 'T';
  } else if (absPerDay >= BILLION) {
    formatted = (Number(absPerDay / 1_000_000n) / 1000).toFixed(2) + 'B';
  } else {
    formatted = (Number(absPerDay) / 1_000_000).toFixed(1) + 'M';
  }

  return `${prefix}${formatted}/day`;
}
```

### Phase 5: Update Modal Chart

**See Spec 5 for definitive chart implementation.** Summary:

1. **Stats panel** shows burn rates with top-up indicators
2. **Trend line** overlays the existing balance histogram

```svelte
<!-- Stats panel in modal -->
<div class="stat-row">
  <span class="stat-label">Short-term Rate</span>
  <span class="stat-value">
    {#if detail.short_term_rate}
      {formatRate(detail.short_term_rate.rate)}
      <span class="meta">
        ({detail.short_term_rate.dataPoints} pts over {detail.short_term_rate.actualHours.toFixed(1)}h,
        R²={detail.short_term_rate.confidence.toFixed(2)})
      </span>
      {#if detail.short_term_rate.topUpActivity !== 'none'}
        <span class="topup-indicator">⚡</span>
      {/if}
    {:else}
      —
    {/if}
  </span>
</div>
```

**Chart trend line** - see Spec 5 for the complete `addTrendLine()` function. Key points:
- Trend line is a dashed overlay on the balance histogram
- Uses same data points as the histogram
- Line endpoints are calculated from `slope * time + intercept`
- Shows visually what the "burn rate" means

---

## Edge Cases and Error Handling

### Edge Case 1: Insufficient Data

```typescript
// If fewer than 2 data points in window
if (points.length < 2) {
  return null;  // UI shows "—" or "Insufficient data"
}
```

### Edge Case 2: Flat Line (No Change)

```typescript
// If all balances are identical, slope = 0
// This is valid! It means no burn.
if (result.slope === 0) {
  return {
    ratePerHour: 0n,
    confidence: 1.0,  // Perfect fit to flat line
    // ...
  };
}
```

### Edge Case 3: Balance Increasing (Top-ups)

**See Decision 4 in Critical Design Decisions for the definitive handling.**

Summary: Top-ups and "negative burn" are the same thing. Detect top-up patterns and handle with `topUpActivity` field:
- `'none'`: Normal regression
- `'single'`: Segment after top-up, show post-top-up rate with ⚡
- `'frequent'`: Show net rate + estimated burn with ⚡⚡

### Edge Case 4: Very Low Confidence (R² < 0.5)

```typescript
// Data is noisy or non-linear
if (result.r2 < 0.5) {
  // Still return the rate, but flag it
  return {
    // ...
    confidence: result.r2,  // UI can show warning icon
  };
}
```

### Edge Case 5: Outlier Detection

For more robust regression, we could implement outlier removal:

```typescript
function robustLinearRegression(points: Array<{t: number, v: number}>): RegressionResult | null {
  // First pass: compute initial regression
  const initial = linearRegression(points);
  if (!initial || points.length < 4) return initial;

  // Calculate residuals and standard deviation
  const residuals = points.map(p => p.v - (initial.slope * p.t + initial.intercept));
  const mean = residuals.reduce((a, b) => a + b, 0) / residuals.length;
  const stdDev = Math.sqrt(
    residuals.reduce((sum, r) => sum + (r - mean) ** 2, 0) / residuals.length
  );

  // Remove points more than 2 standard deviations from fit
  const filtered = points.filter((p, i) => Math.abs(residuals[i] - mean) <= 2 * stdDev);

  // If we removed more than 25% of points, use original (data might be legitimately noisy)
  if (filtered.length < points.length * 0.75) return initial;

  // Recompute with filtered data
  return linearRegression(filtered);
}
```

---

## Critical Design Decisions

> **Note:** This section documents the reasoning behind decisions. For the **definitive implementation specifications** with code examples, see the "Implementation Specifications" section at the end of this document.

This section addresses specific concerns that MUST be handled correctly during implementation.

### Decision 1: Window Size vs Rate Period (IMPORTANT)

**The Problem:**
```typescript
// CONFUSING - Why 2 hours for "hourly"?
hourly: calculateBurnRate(snapshots, 2 * HOUR, now, canisterId),
daily: calculateBurnRate(snapshots, 36 * HOUR, now, canisterId),
```

This conflates two distinct concepts:
- **Window size**: How much historical data feeds the regression
- **Rate period**: What time unit we express the result in (always cycles/hour)

**The Solution:**

Rename columns to reflect what they actually represent:

| Old Name | New Name | Window Size | What It Means |
|----------|----------|-------------|---------------|
| "Hourly Rate" | "Recent" | 2 hours | Trend from very recent data |
| "Daily Rate" | "Short-term" | 36 hours | Trend from ~1 day of data |
| "Weekly Rate" | "Long-term" | 7 days | Trend from all available data |

**Updated Interface:**

```typescript
export interface BurnRates {
  recent: {
    rate: bigint;           // cycles per hour
    confidence: number;     // R²
    dataPoints: number;     // snapshots used
    windowHours: number;    // actual time span (for display)
  } | null;

  shortTerm: {
    rate: bigint;
    confidence: number;
    dataPoints: number;
    windowHours: number;
  } | null;

  longTerm: {
    rate: bigint;
    confidence: number;
    dataPoints: number;
    windowHours: number;
  } | null;
}
```

**UI Display:**
```svelte
<th>
  Recent
  <span class="sublabel">(~2h window)</span>
</th>
<th>
  Short-term
  <span class="sublabel">(~24h window)</span>
</th>
<th>
  Long-term
  <span class="sublabel">(~7d window)</span>
</th>
```

All columns show the same unit (cycles/hour or cycles/day) - the difference is how much history informs the trend calculation.

---

### Decision 2: Numeric Precision Safety

**The Concern:**
```typescript
v: Number(BigInt(balanceStr))  // Is this safe?
```

**Analysis:**
- `Number.MAX_SAFE_INTEGER` = 9,007,199,254,740,991 (~9 × 10¹⁵)
- Largest canister balances: ~100T = 10¹⁴ cycles
- **Safe margin: 90x** - We're fine.

**Implementation Requirement:**
Add an explicit comment and safety check:

```typescript
function safeBalanceToNumber(balanceStr: string): number {
  const balance = BigInt(balanceStr);

  // Safety check: Number.MAX_SAFE_INTEGER ≈ 9×10^15
  // Largest expected balance ≈ 100T = 10^14
  // This gives us 90x safety margin
  if (balance > 9_000_000_000_000_000n) {
    console.warn(`Balance ${balanceStr} exceeds safe integer range, precision may be lost`);
  }

  return Number(balance);
}
```

---

### Decision 3: Low R² Is Not Always Bad

**The Concern:**
A canister with genuinely spiky compute (e.g., batch processing) will have low R² because burn varies naturally. The slope is still the best linear estimate - low R² just means "not perfectly linear".

**Implementation Requirement:**

1. **Always calculate and return the rate**, regardless of R²
2. **Style based on R² but never hide the data**
3. **Explain what R² means in tooltips**

```typescript
// DO NOT DO THIS:
if (result.r2 < 0.5) return null;  // WRONG - hides useful data

// DO THIS INSTEAD:
return {
  rate: ...,
  confidence: result.r2,  // Let UI decide how to display
  isLinear: result.r2 > 0.8,  // Helper flag for styling
};
```

**UI Styling:**

```svelte
<td class="rate">
  <span
    class="rate-value"
    class:high-confidence={rate.confidence > 0.8}
    class:medium-confidence={rate.confidence > 0.5 && rate.confidence <= 0.8}
    class:low-confidence={rate.confidence <= 0.5}
    title={`Confidence: ${(rate.confidence * 100).toFixed(0)}% (R²)\n` +
           `${rate.dataPoints} data points over ${rate.windowHours.toFixed(1)}h\n` +
           (rate.confidence < 0.5 ? 'Low R² may indicate variable/spiky burn pattern' : '')}
  >
    {formatRate(rate.rate)}
  </span>
</td>

<style>
  .high-confidence { color: var(--text); }
  .medium-confidence { color: var(--text); opacity: 0.85; }
  .low-confidence {
    color: var(--text-muted);
    font-style: italic;
  }
  .low-confidence::after {
    content: " ~";  /* Indicates approximation */
  }
</style>
```

---

### Decision 4: Top-Up Handling (Unified with Negative Burn)

**Key Insight:** Negative burn and top-up are THE SAME THING. The only way a canister's balance increases is by receiving cycles. There is no "natural" balance increase.

**Scenarios:**

1. **Single top-up:** One large jump, then normal burning resumes
2. **Frequent top-ups:** Canister receives cycles regularly (automated management)
3. **Net gaining:** Top-ups exceed burn rate consistently

**Implementation Requirement:**

Handle all cases with a unified approach:

```typescript
interface BurnRate {
  rate: bigint;              // Positive = burning, Negative = net gaining
  confidence: number;        // R²
  dataPoints: number;
  windowHours: number;
  topUpActivity: 'none' | 'single' | 'frequent';  // What kind of top-up pattern
  totalTopUps: bigint;       // Sum of all detected top-ups in window
  netBurnRate: bigint;       // Burn rate excluding top-ups (if calculable)
}
```

**Detection Logic:**

```typescript
function analyzeTopUps(points: Array<{t: number, v: number}>): {
  activity: 'none' | 'single' | 'frequent';
  topUps: Array<{index: number, amount: number}>;
  totalAmount: number;
} {
  const sorted = [...points].sort((a, b) => a.t - b.t);
  const topUps: Array<{index: number, amount: number}> = [];

  // Detect ALL positive jumps > threshold
  const THRESHOLD = 100_000_000_000;  // 100B (smaller threshold to catch more)

  for (let i = 1; i < sorted.length; i++) {
    const jump = sorted[i].v - sorted[i-1].v;
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
```

**Calculation Strategy by Pattern:**

```typescript
function calculateBurnRate(points, ...): BurnRate {
  const analysis = analyzeTopUps(points);
  const sorted = [...points].sort((a, b) => a.t - b.t);

  switch (analysis.activity) {
    case 'none': {
      // Simple case: no top-ups, just run regression
      const result = linearRegression(points);
      return {
        rate: ...,
        topUpActivity: 'none',
        totalTopUps: 0n,
        netBurnRate: result.rate,  // Same as rate
      };
    }

    case 'single': {
      // One top-up: segment after it for accurate burn rate
      const lastTopUp = analysis.topUps[analysis.topUps.length - 1];
      const postTopUpPoints = sorted.slice(lastTopUp.index);

      if (postTopUpPoints.length >= 2) {
        const result = linearRegression(postTopUpPoints);
        return {
          rate: result.rate,  // Burn rate after top-up
          topUpActivity: 'single',
          totalTopUps: BigInt(Math.round(analysis.totalAmount)),
          netBurnRate: result.rate,
        };
      }
      // Fall through if not enough post-top-up data
    }

    case 'frequent': {
      // Frequent top-ups: can't segment, report net rate AND estimated burn
      const result = linearRegression(points);

      // Net rate (includes top-ups) - may be positive (gaining) or negative (burning faster than top-ups)
      const netRate = -result.slope * 3600000;

      // Estimate "pure" burn by looking at intervals WITHOUT top-ups
      const burnIntervals = [];
      for (let i = 1; i < sorted.length; i++) {
        const change = sorted[i].v - sorted[i-1].v;
        if (change <= 0) {  // Only burning intervals (no top-up)
          const duration = sorted[i].t - sorted[i-1].t;
          burnIntervals.push({ burn: -change, duration });
        }
      }

      let estimatedBurnRate = 0n;
      if (burnIntervals.length > 0) {
        const totalBurn = burnIntervals.reduce((s, i) => s + i.burn, 0);
        const totalDuration = burnIntervals.reduce((s, i) => s + i.duration, 0);
        estimatedBurnRate = BigInt(Math.round((totalBurn / totalDuration) * 3600000));
      }

      return {
        rate: BigInt(Math.round(netRate)),  // Net rate (may be negative = gaining)
        topUpActivity: 'frequent',
        totalTopUps: BigInt(Math.round(analysis.totalAmount)),
        netBurnRate: estimatedBurnRate,  // Burn rate from non-top-up intervals
      };
    }
  }
}
```

**UI Display - Unified:**

```svelte
<td class="rate">
  {#if rate === null}
    <span class="no-data">—</span>

  {:else if rate.topUpActivity === 'none'}
    <!-- Normal burning, no top-ups -->
    {#if rate.rate < 0n}
      <!-- This shouldn't happen without top-ups, but handle gracefully -->
      <span class="rate-value gaining">↑ {formatRate(-rate.rate)}/day</span>
    {:else}
      <span class="rate-value">{formatRate(rate.rate)}/day</span>
    {/if}

  {:else if rate.topUpActivity === 'single'}
    <!-- Single top-up detected, showing post-top-up burn rate -->
    <span class="rate-value">
      {formatRate(rate.rate)}/day
      <span class="topup-badge" title="Top-up of {formatCycles(rate.totalTopUps)} detected. Rate is post-top-up.">
        ⚡
      </span>
    </span>

  {:else if rate.topUpActivity === 'frequent'}
    <!-- Frequent top-ups, show both net and burn -->
    <span class="rate-value frequent-topups">
      {#if rate.rate < 0n}
        <span class="net-gaining">↑ {formatRate(-rate.rate)}/day net</span>
      {:else}
        <span class="net-burning">{formatRate(rate.rate)}/day net</span>
      {/if}
      <span class="burn-estimate" title="Estimated burn rate from intervals without top-ups">
        (burns ~{formatRate(rate.netBurnRate)}/day)
      </span>
      <span class="topup-badge" title="Frequent top-ups detected ({formatCycles(rate.totalTopUps)} total)">
        ⚡⚡
      </span>
    </span>
  {/if}
</td>
```

**Example Displays:**

```
| Canister  | Balance | Recent                          |
|-----------|---------|----------------------------------|
| normal    | 50T     | 1.2T/day                        |  ← No top-ups
| topped-up | 100T    | 0.8T/day ⚡                      |  ← Single top-up, rate is post-top-up
| managed   | 75T     | ↑ 2T/day net (burns ~5T/day) ⚡⚡ |  ← Frequent top-ups
| draining  | 30T     | 3T/day net (burns ~3T/day) ⚡⚡   |  ← Top-ups < burn
```

**Edge Case: Topped Up Every Hour**

If literally every interval has a top-up:
- `burnIntervals` would be empty (no pure-burn intervals)
- `netBurnRate` would be 0 or null
- UI shows: "↑ Xday net (burn rate unknown) ⚡⚡"

```typescript
if (burnIntervals.length === 0) {
  return {
    rate: netRate,
    topUpActivity: 'frequent',
    totalTopUps: ...,
    netBurnRate: null,  // Can't calculate - no burn-only intervals
  };
}
```

```svelte
{#if rate.netBurnRate === null}
  <span class="burn-estimate unknown">
    (burn rate unknown - constant top-ups)
  </span>
{/if}
```

---

### Decision 5: Net Rate Display (Replaces "Negative Burn")

**Clarification:** Since top-ups are the only source of balance increase, "negative burn" is just another way of saying "net gaining due to top-ups". We handle this in Decision 4 above.

**Key Points:**
1. **Preserve the sign** - don't clamp to zero
2. **Distinguish net rate from burn rate** when there are top-ups
3. **Use clear language** - "net" for overall, "burns" for consumption

**Color Coding:**
```css
.net-gaining { color: var(--blue); }     /* Balance trending up */
.net-burning { color: var(--text); }      /* Balance trending down (normal) */
.burn-estimate { color: var(--text-muted); font-size: 0.85em; }
.topup-badge { color: var(--orange); }
```

---

### Decision 6: Show Actual Time Spans

**The Concern:**
Users should know exactly what data period informed each calculation.

**Implementation Requirement:**

Always show the actual time span, not the target window:

```typescript
interface BurnRate {
  rate: bigint;
  confidence: number;
  dataPoints: number;
  actualHours: number;   // Actual span of data used
  targetHours: number;   // What we asked for (2h, 36h, 168h)
}
```

**UI Display:**
```svelte
<th>
  Recent
  {#if stats.recentRate?.actualHours}
    <span class="time-delta">({formatTimeDelta(stats.recentRate.actualHours)})</span>
  {/if}
</th>
```

**Example:**
```
| Recent (1.8h) | Short-term (23.5h) | Long-term (6.2d) |
```

This makes it explicit: "Recent rate is calculated from 1.8 hours of data."

---

## Summary of Decisions

| Concern | Decision | Implementation |
|---------|----------|----------------|
| Window vs Rate confusion | Rename to Recent/Short-term/Long-term | Update interfaces and UI labels |
| Numeric precision | Safe for our use case | Add comment + warning for edge cases |
| Low R² handling | Always show rate, style by confidence | Never hide, use visual hierarchy |
| Top-ups & Negative burn | **Unified handling** - detect pattern, adapt calculation | Three modes: none/single/frequent |
| Single top-up | Segment after top-up | Show burn rate with ⚡ indicator |
| Frequent top-ups | Report net rate + estimated burn | Show both with ⚡⚡ indicator |
| Every-interval top-ups | Report net rate, mark burn as unknown | "burn rate unknown" message |
| Time spans | Show actual, not target | Add actualHours to interface |

---

## UI Display Recommendations

### Main Table

| Column | Display | Tooltip |
|--------|---------|---------|
| Recent (~2h) | "1.2T/day" | "Calculated from {n} snapshots over {actualHours}h. R²={confidence}" |
| Short-term (~24h) | "28.5T/day" | Same format |
| Long-term (~7d) | "195T/day" | Same format |

All rates displayed as `/day` - see Spec 1.

### Confidence Indicators

```css
/* High confidence (R² > 0.8) - normal display */
.rate-value { color: var(--text); }

/* Medium confidence (0.5 < R² < 0.8) - subtle warning */
.rate-value.medium-confidence { color: var(--orange); }

/* Low confidence (R² < 0.5) - clear warning */
.rate-value.low-confidence {
  color: var(--text-muted);
  font-style: italic;
}
.rate-value.low-confidence::after {
  content: " ⚠";
  font-size: 0.8em;
}
```

### Stats Header

```
Tracking 2,898 canisters across 155 projects
Last snapshot: 14 minutes ago | 52 snapshots over 2.3 days | Avg interval: 62 min
```

---

## Migration Path

### Step 1: Add Regression Module (No UI Changes)
- Create `regression.ts` with calculation functions
- Add tests to verify correctness
- No user-visible changes yet

### Step 2: Dual Calculation (Shadow Mode)
- Calculate both old (two-point) and new (regression) values
- Log discrepancies to console for analysis
- Still display old values to users

### Step 3: Update Data Structures
- Add new rate fields to interfaces
- Populate both old and new fields
- UI still uses old fields

### Step 4: Update UI Components
- Switch displays to use new rate fields
- Update column headers and tooltips
- Add confidence indicators

### Step 5: Remove Old Code
- Remove `findSnapshotNearTime` function
- Remove old burn fields from interfaces
- Clean up unused code

### Step 6: Polish
- Tune confidence thresholds based on real data
- Consider adding trend arrows (↑↓→) for quick visual scanning

---

## Testing Plan

### Unit Tests for Regression

```typescript
describe('linearRegression', () => {
  it('returns null for fewer than 2 points', () => {
    expect(linearRegression([])).toBeNull();
    expect(linearRegression([{t: 0, v: 100}])).toBeNull();
  });

  it('calculates correct slope for perfect linear data', () => {
    const points = [
      {t: 0, v: 100},
      {t: 1, v: 90},
      {t: 2, v: 80},
    ];
    const result = linearRegression(points);
    expect(result?.slope).toBe(-10);
    expect(result?.r2).toBe(1);
  });

  it('handles flat line (no change)', () => {
    const points = [
      {t: 0, v: 100},
      {t: 1, v: 100},
      {t: 2, v: 100},
    ];
    const result = linearRegression(points);
    expect(result?.slope).toBe(0);
  });

  it('handles noisy data with reduced R²', () => {
    const points = [
      {t: 0, v: 100},
      {t: 1, v: 95},
      {t: 2, v: 85},  // Should be 90 for perfect line
      {t: 3, v: 75},
    ];
    const result = linearRegression(points);
    expect(result?.slope).toBeCloseTo(-8.5, 1);
    expect(result?.r2).toBeLessThan(1);
    expect(result?.r2).toBeGreaterThan(0.9);
  });
});
```

### Integration Tests

```typescript
describe('calculateBurnRate', () => {
  it('calculates correct burn rate from real snapshot format', () => {
    const snapshots = [
      { timestamp: 3600000, balances: { 'abc': '90000000000000' } },
      { timestamp: 0, balances: { 'abc': '100000000000000' } },
    ];
    const result = calculateBurnRate(snapshots, 7200000, 3600000, 'abc');
    expect(result?.ratePerHour).toBe(10000000000000n); // 10T/hour
  });
});
```

---

## Implementation Specifications

These are **definitive decisions**, not open questions. Follow these exactly.

### Spec 1: Storage vs Display Units

**Store:** cycles per millisecond (raw slope from regression)
**Display:** cycles per day (human-readable)

The conversion happens ONLY in the display layer:

```typescript
// In regression.ts - return raw slope
export interface RegressionResult {
  slope: number;           // cycles per millisecond (raw)
  // ...
}

// In regression.ts - BurnRate stores per-hour for intermediate use
export interface BurnRate {
  ratePerHour: bigint;     // cycles per hour (slope × 3,600,000)
  // ...
}

// In UI components - format for display
function formatRate(ratePerHour: bigint): string {
  const perDay = ratePerHour * 24n;  // Convert to per-day HERE
  // ... format as T/day, B/day, M/day
}
```

**All UI displays use `/day` suffix.** No `/hr` anywhere in the UI.

---

### Spec 2: Simplified Data Types (Fix Circular Calculation)

The Phase 3 code was wrong. Here's the correct, simplified version:

```typescript
// CanisterEntry stores ONE rate per window, always as cycles/hour
export interface CanisterEntry {
  canister_id: string;
  project: string[] | null;
  balance: bigint;
  valid: boolean;

  // All rates stored as cycles/hour (display layer converts to /day)
  recent_rate: BurnRateData | null;    // ~2h window
  short_term_rate: BurnRateData | null; // ~36h window
  long_term_rate: BurnRateData | null;  // ~7d window
}

export interface BurnRateData {
  rate: bigint;             // cycles per hour (positive = burning)
  confidence: number;       // R² (0-1)
  dataPoints: number;       // snapshots used
  actualHours: number;      // actual time span of data
  topUpActivity: 'none' | 'single' | 'frequent';
  netBurnRate: bigint | null;  // burn rate excluding top-ups (null if uncalculable)
}
```

No more redundant `ratePerHour` / `ratePerDay` in the same object.

---

### Spec 3: Project R² Aggregation

R² doesn't sum. For project aggregates, use **weighted average by data points**:

```typescript
export interface ProjectRateData {
  rate: bigint;                    // Sum of all canister rates
  avgConfidence: number;           // Weighted average R²
  totalDataPoints: number;         // Sum of all data points
  canistersWithData: number;       // How many canisters contributed
  canistersWithLowConfidence: number;  // How many had R² < 0.5
}

// Calculation:
function aggregateProjectRate(canisterRates: BurnRateData[]): ProjectRateData {
  const validRates = canisterRates.filter(r => r !== null);
  if (validRates.length === 0) return null;

  const totalRate = validRates.reduce((sum, r) => sum + r.rate, 0n);
  const totalPoints = validRates.reduce((sum, r) => sum + r.dataPoints, 0);

  // Weighted average: Σ(R² × dataPoints) / Σ(dataPoints)
  const weightedR2Sum = validRates.reduce(
    (sum, r) => sum + r.confidence * r.dataPoints, 0
  );
  const avgConfidence = weightedR2Sum / totalPoints;

  return {
    rate: totalRate,
    avgConfidence,
    totalDataPoints: totalPoints,
    canistersWithData: validRates.length,
    canistersWithLowConfidence: validRates.filter(r => r.confidence < 0.5).length,
  };
}
```

**UI display for projects:**
```svelte
<td>
  {formatRate(project.short_term_rate.rate)}/day
  {#if project.short_term_rate.canistersWithLowConfidence > 0}
    <span class="warning" title="{project.short_term_rate.canistersWithLowConfidence} canisters have low confidence">
      ({project.short_term_rate.canistersWithLowConfidence} ⚠)
    </span>
  {/if}
</td>
```

---

### Spec 4: Sorting Behavior

**Sort by rate value descending. Nulls sort to bottom.**

```typescript
// Sort function for canister table
function sortByRate(a: CanisterEntry, b: CanisterEntry, column: 'recent' | 'short_term' | 'long_term'): number {
  const aRate = a[`${column}_rate`]?.rate ?? null;
  const bRate = b[`${column}_rate`]?.rate ?? null;

  // Nulls go to bottom
  if (aRate === null && bRate === null) return 0;
  if (aRate === null) return 1;   // a goes after b
  if (bRate === null) return -1;  // b goes after a

  // Sort by absolute rate descending (highest burn first)
  // Negative rates (gaining) sort below positive rates (burning)
  if (bRate > aRate) return 1;
  if (bRate < aRate) return -1;
  return 0;
}
```

**Confidence does NOT affect sort order.** It only affects display styling.

**Zero vs Null distinction:**
- `null` = insufficient data to calculate (< 2 snapshots)
- `0n` = calculated rate is zero (canister not burning)

Zero sorts normally (above negatives, below positives). Null sorts to bottom.

---

### Spec 5: Chart Integration

The trend line goes on the **existing balance chart** as an overlay. No separate burn chart.

**Implementation details:**

```typescript
// In CanisterDetailModal.svelte

// 1. Chart shows balance over time (existing histogram bars)
// 2. Overlay a dashed trend line showing the regression fit

function addTrendLine(chart: IChartApi, snapshots: Snapshot[], canisterId: string) {
  // Get points for regression (same as balance chart data)
  const points = snapshots
    .filter(s => s.balances[canisterId])
    .map(s => ({
      t: s.timestamp,
      v: Number(BigInt(s.balances[canisterId])),
    }));

  if (points.length < 2) return;

  const result = linearRegression(points);
  if (!result) return;

  // Create line series
  const lineSeries = chart.addLineSeries({
    color: 'rgba(255, 107, 107, 0.7)',  // Semi-transparent red
    lineWidth: 2,
    lineStyle: 2,  // Dashed
    lastValueVisible: false,
    priceLineVisible: false,
  });

  // Generate trend line from first to last point
  const sortedPoints = [...points].sort((a, b) => a.t - b.t);
  const firstTime = sortedPoints[0].t;
  const lastTime = sortedPoints[sortedPoints.length - 1].t;

  const trendData = [
    {
      time: Math.floor(firstTime / 1000) as UTCTimestamp,
      value: (result.slope * firstTime + result.intercept) / 1e12,  // Convert to T
    },
    {
      time: Math.floor(lastTime / 1000) as UTCTimestamp,
      value: (result.slope * lastTime + result.intercept) / 1e12,
    },
  ];

  lineSeries.setData(trendData);
}

// Call after creating histogram:
// const histogramSeries = chart.addHistogramSeries(...);
// histogramSeries.setData(chartData);
// addTrendLine(chart, snapshots, canisterId);  // ← Add this
```

**Visual result:** Balance bars with a dashed line showing the overall trend. The slope of that line IS the burn rate.

---

### Spec 6: Global Stats Header

The stats header shows **sum of all canister burn rates**.

```typescript
export interface GlobalStats {
  canisterCount: number;
  projectCount: number;
  lastUpdated: Date | null;
  snapshotCount: number;

  // Aggregated burn rates across ALL canisters
  totalBurnRate: {
    recent: bigint | null;      // Sum of all recent rates
    shortTerm: bigint | null;   // Sum of all short-term rates
    longTerm: bigint | null;    // Sum of all long-term rates
    avgConfidence: number;      // Weighted average R² across all
  };
}

// Calculation in loadData():
const allRates = entries.map(e => e.short_term_rate).filter(r => r !== null);
const totalBurnRate = allRates.reduce((sum, r) => sum + r.rate, 0n);
const totalPoints = allRates.reduce((sum, r) => sum + r.dataPoints, 0);
const avgConfidence = allRates.reduce((sum, r) => sum + r.confidence * r.dataPoints, 0) / totalPoints;
```

**Header display:**

```svelte
<div class="stats-header">
  <span>Tracking {stats.canisterCount} canisters</span>
  <span class="separator">|</span>
  <span>
    Network burn: {formatRate(stats.totalBurnRate.shortTerm)}/day
    {#if stats.totalBurnRate.avgConfidence < 0.7}
      <span class="low-confidence" title="Average R² = {stats.totalBurnRate.avgConfidence.toFixed(2)}">~</span>
    {/if}
  </span>
  <span class="separator">|</span>
  <span>Last update: {formatTimeAgo(stats.lastUpdated)}</span>
</div>
```

---

## Conclusion

Linear regression provides a mathematically sound, computationally efficient solution to our irregular data problem. Instead of pretending we have precise hourly snapshots, we embrace the reality of our data and extract the meaningful signal: **the rate of change**.

Benefits:
- Uses ALL available data, not just two arbitrary points
- Naturally handles irregular timing
- Provides confidence metric (R²)
- Honest labels ("Burn Rate" vs "1h Burn")
- Trivial performance impact (~11ms for all calculations)

The implementation is straightforward and can be done incrementally without breaking existing functionality.
