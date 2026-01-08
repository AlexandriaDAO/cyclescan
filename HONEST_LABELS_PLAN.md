# CycleScan: Honest Labels Approach

## The Goal

Accurately attribute the network's daily cycle burn (~$5,000/day) to specific projects, so the community can see who's contributing the most.

## The Problem (Restated)

We have hourly snapshots with timing variance. Our current UI says:
- "1h Burn: 50T" — but the actual measurement window might be 30-90 minutes
- "24h Burn: 500T" — but the actual window might be 22-26 hours

The numbers are approximately right, but the labels are lies.

## The Solution: Honest Labels

**Show exactly what we measured, with the actual time window.**

### Display Format

Instead of:
```
| Project | 1h Burn | 24h Burn | 7d Burn |
|---------|---------|----------|---------|
| OpenChat|   5T    |   120T   |  800T   |
```

Show:
```
| Project | Recent      | ~24h Burn    | ~7d Burn     |
|---------|-------------|--------------|--------------|
| OpenChat| 5T (52m)    | 120T (23.5h) | 800T (6.9d)  |
```

### Key Changes

1. **Rename "1h Burn" → "Recent"**
   - Shows burn since the previous snapshot
   - Time delta tells you how recent
   - No pretense of measuring exactly 1 hour

2. **Add "~" to 24h and 7d headers**
   - Visual cue that these are approximate windows
   - The actual time in parentheses gives the truth

3. **Always show actual time delta**
   - "120T (23.5h)" = "120 trillion cycles burned over the last 23.5 hours"
   - User can verify: 120T / 23.5h ≈ 5.1T/hour
   - No magic, no hidden calculations

---

## Detailed Design

### Data Structure Changes

```typescript
// Current
export interface CanisterEntry {
  burn_1h: [bigint] | [];      // Just the amount
  burn_24h: [bigint] | [];
  burn_7d: [bigint] | [];
}

// Proposed
export interface BurnMeasurement {
  amount: bigint;          // Cycles burned (positive = burned, negative = gained)
  actualHours: number;     // Actual time span in hours
  snapshotAge: number;     // How old is the comparison snapshot (ms)
}

export interface CanisterEntry {
  recent_burn: BurnMeasurement | null;   // Since last snapshot
  daily_burn: BurnMeasurement | null;    // ~24h window
  weekly_burn: BurnMeasurement | null;   // ~7d window
}
```

### Time Window Logic

```typescript
// Find snapshot closest to target time
// Return both the burn amount AND the actual time delta

function measureBurn(
  snapshots: Snapshot[],
  canisterId: string,
  targetHoursAgo: number,
  toleranceRatio: number = 0.5
): BurnMeasurement | null {
  const now = snapshots[0].timestamp;
  const targetTime = now - (targetHoursAgo * 3600000);
  const tolerance = targetHoursAgo * 3600000 * toleranceRatio;

  // Find closest snapshot to target time
  const compareSnapshot = findSnapshotNearTime(snapshots, targetTime, tolerance);
  if (!compareSnapshot) return null;

  const currentBalance = BigInt(snapshots[0].balances[canisterId] || '0');
  const previousBalance = BigInt(compareSnapshot.balances[canisterId] || '0');

  const actualMs = now - compareSnapshot.timestamp;
  const actualHours = actualMs / 3600000;

  return {
    amount: previousBalance - currentBalance,  // Positive = burned
    actualHours: actualHours,
    snapshotAge: actualMs,
  };
}
```

### Display Logic

```typescript
function formatBurnWithDelta(burn: BurnMeasurement | null): string {
  if (!burn) return '—';

  const amount = formatCycles(burn.amount);  // "120T"
  const hours = burn.actualHours;

  // Format time delta appropriately
  let timeStr: string;
  if (hours < 1) {
    timeStr = `${Math.round(hours * 60)}m`;     // "52m"
  } else if (hours < 24) {
    timeStr = `${hours.toFixed(1)}h`;           // "23.5h"
  } else {
    timeStr = `${(hours / 24).toFixed(1)}d`;    // "6.9d"
  }

  // Handle negative burn (balance increased)
  if (burn.amount < 0n) {
    return `↑${formatCycles(-burn.amount)} (${timeStr})`;  // "↑5T (24h)"
  }

  return `${amount} (${timeStr})`;  // "120T (23.5h)"
}
```

---

## Table Column Design

### Headers

```
| # | Project | Cans | Balance | Recent | ~24h Burn | ~7d Burn |
```

- **Recent**: Burn since last snapshot (replaces "1h Burn")
- **~24h Burn**: Approximate 24-hour window
- **~7d Burn**: Approximate 7-day window

### Header Tooltips

- **Recent**: "Cycles burned since the previous snapshot"
- **~24h Burn**: "Cycles burned over approximately the last 24 hours. Actual measurement window shown in parentheses."
- **~7d Burn**: "Cycles burned over approximately the last 7 days."

### Cell Format

```
| 1 | OpenChat | 12 | 500T | 5T (52m) | 120T (23.5h) | 800T (6.9d) |
```

The time delta in parentheses serves multiple purposes:
1. Shows the actual measurement window
2. Acts as a data quality indicator (23.5h is good; 18h might be a gap)
3. Lets users calculate their own rates if desired

---

## Handling Edge Cases

### 1. Missing Snapshot (No Data)

If no snapshot exists within tolerance:
```
~24h Burn: —
```

Tooltip: "No snapshot available near the 24-hour mark"

### 2. Balance Increased (Negative Burn)

When a canister receives cycles:
```
~24h Burn: ↑5T (24h)
```

- Different styling (blue/gray instead of green)
- Up arrow indicates balance increased
- Tooltip: "This canister gained 5T cycles over the last 24 hours"

### 3. Zero Burn

When balance is unchanged:
```
~24h Burn: 0 (24h)
```

Shows the canister is tracked but not burning.

### 4. Very Old Data

If the nearest snapshot is unusually far from target:
```
~24h Burn: 100T (31h) ⚠
```

Warning indicator when actualHours deviates significantly from target.
Threshold: >25% deviation (e.g., <18h or >30h for 24h target)

---

## Sorting Behavior

### Primary Sort: Raw Amount (Default)

Sort by `burn.amount`, not by normalized rate.

Why: "Who burned the most cycles?" is the core question. A project that burned 100T over 20 hours burned MORE than one that burned 90T over 24 hours, even though the latter has a higher rate.

### Alternative: Rate-Normalized Sort (Optional)

Could add as user preference:
```
Sort by: [Raw amount ▼] [Daily rate]
```

Rate calculation: `amount / actualHours * 24`

But raw amount should be default — it's what actually happened.

---

## Stats Header Update

### Current
```
Tracking 2,898 canisters across 155 projects
```

### Proposed
```
Tracking 2,898 canisters | Last snapshot: 14 min ago | 168 snapshots (7.1 days)
```

Additional context about data freshness and coverage.

---

## Modal (Canister Detail) Updates

### Stats Panel

Current:
```
Current Balance    500T
1h Burn            5T
24h Burn           120T
7d Burn            800T
```

Proposed:
```
Current Balance    500T
Recent Burn        5T (52 minutes ago)
~24h Burn          120T over 23.5 hours
~7d Burn           800T over 6.9 days
```

### Chart

No changes needed. The chart already shows actual balance at actual timestamps. This is correct behavior.

---

## Project Aggregation

### Current Approach
Sum all canister burns for the project.

### With Honest Labels

Same summation, but time deltas may vary per canister:
- Canister A: 50T (23.5h)
- Canister B: 30T (24.1h)
- Canister C: 40T (22.8h)
- **Project Total: 120T**

What time delta to show for the project total?

**Option 1: Weighted Average**
```
avgHours = Σ(amount × hours) / Σ(amount)
         = (50×23.5 + 30×24.1 + 40×22.8) / 120
         = 23.4h
Display: "120T (23.4h)"
```

**Option 2: Range**
```
Display: "120T (22.8-24.1h)"
```

**Option 3: Just the sum, no time**
```
Display: "120T (~24h)"
```

Recommendation: **Option 1 (weighted average)** — it's the most honest single number.

---

## Implementation Plan

### Phase 1: Data Layer (data.ts)

1. Add `BurnMeasurement` interface
2. Update `measureBurn()` to return actual time delta
3. Update `CanisterEntry` and `ProjectEntry` types
4. Update `loadData()` to compute new structure

### Phase 2: Table Display (+page.svelte)

1. Update `formatBurn()` → `formatBurnWithDelta()`
2. Rename column headers (1h → Recent, add ~ prefix)
3. Add tooltips to headers
4. Handle negative burn display
5. Add warning indicator for large time deviations

### Phase 3: Modal (CanisterDetailModal.svelte)

1. Update stats panel text
2. Add time context to burn values
3. (No chart changes needed)

### Phase 4: Polish

1. Add header tooltips explaining methodology
2. Consider adding "Data Quality" indicator
3. Update any documentation

---

## What This Achieves

✅ **100% Accurate** — We show exactly what we measured
✅ **No Arbitrary Thresholds** — No R², no confidence intervals
✅ **Spikes Preserved** — Real changes visible, not smoothed
✅ **Simple to Verify** — User can check the math
✅ **Transparent About Uncertainty** — Time delta IS the uncertainty
✅ **Minimal Code Changes** — Evolution, not revolution

---

## What This Doesn't Solve

❌ **Can't make 1h data more reliable** — With hourly collection, we can only measure ~1h windows approximately. The "Recent" rename is honest about this.

❌ **Doesn't extrapolate** — We don't compute "daily rate from hourly data". We measure what we measure.

❌ **Doesn't smooth outliers** — A spike day shows as a spike. This is a feature, not a bug.

---

## Table Header Time Deltas

Since all canisters compare against the **same reference snapshots**, the time delta is identical for all rows. Put it in the header, not each cell:

```
| # | Project | Cans | Balance | Recent (52m) | ~24h (23.5h) | ~7d (6.9d) |
|---|---------|------|---------|--------------|--------------|------------|
| 1 | OpenChat| 12   | 500T    | 5T           | 120T         | 800T       |
| 2 | Taggr   | 8    | 200T    | 2T           | 45T          | 310T       |
```

The header tells you "this column measures the ~24 hour window, which was actually 23.5 hours for this data load."

---

## Modal Chart Design: Hybrid Interpolated Burn Chart

### The Concept

Show **two layers** of data:

1. **Raw data points (subtle)** — Actual measurements at actual times
2. **Interpolated hourly bars (prominent)** — Normalized for easy reading

This gives both **honesty** (raw data visible) and **usability** (easy hour-by-hour comparison).

### Visual Design

```
Burn (T)
│
3├                    ○ (raw: 3T over 90min)
 │
2├              ████████
 │        ██████
1├──○─────██              (raw: 1T over 30min)
 │  ████
 └──┬─────┬─────┬─────▶
   8:00  9:00  10:00  11:00

○ = actual measurement point (subtle, semi-transparent)
█ = interpolated hourly burn (prominent bars)
```

### How Interpolation Works

Given raw measurements that span irregular time periods, distribute burn proportionally across standard hourly buckets.

**Example:**
- Measurement A: 1T burned from 8:15-8:45 (30 min)
- Measurement B: 3T burned from 8:45-10:15 (90 min)

**Interpolated hourly values:**
```
Hour 8-9:
  - All of measurement A (30 min): 1T
  - Part of measurement B (15 min of 90 min): 3T × (15/90) = 0.5T
  - Total: 1.5T

Hour 9-10:
  - Part of measurement B (60 min of 90 min): 3T × (60/90) = 2T
  - Total: 2T

Hour 10-11:
  - Part of measurement B (15 min of 90 min): 3T × (15/90) = 0.5T
  - Total: 0.5T
```

### Algorithm

```typescript
interface BurnDelta {
  startTime: number;    // ms timestamp
  endTime: number;      // ms timestamp
  burnAmount: bigint;   // cycles burned
}

interface HourlyBurn {
  hourStart: number;    // ms timestamp (start of hour)
  burnAmount: number;   // interpolated cycles burned in this hour
}

function interpolateToHourlyBuckets(deltas: BurnDelta[]): HourlyBurn[] {
  const HOUR_MS = 3600000;
  const hourlyBurns = new Map<number, number>();

  for (const delta of deltas) {
    const durationMs = delta.endTime - delta.startTime;
    if (durationMs <= 0) continue;

    const burnPerMs = Number(delta.burnAmount) / durationMs;

    // Find all hours this measurement spans
    const startHour = Math.floor(delta.startTime / HOUR_MS) * HOUR_MS;
    const endHour = Math.floor(delta.endTime / HOUR_MS) * HOUR_MS;

    for (let hour = startHour; hour <= endHour; hour += HOUR_MS) {
      const hourEnd = hour + HOUR_MS;

      // Calculate overlap between measurement and this hour
      const overlapStart = Math.max(delta.startTime, hour);
      const overlapEnd = Math.min(delta.endTime, hourEnd);
      const overlapMs = Math.max(0, overlapEnd - overlapStart);

      if (overlapMs > 0) {
        const burnInThisHour = burnPerMs * overlapMs;
        hourlyBurns.set(hour, (hourlyBurns.get(hour) || 0) + burnInThisHour);
      }
    }
  }

  // Convert to sorted array
  return Array.from(hourlyBurns.entries())
    .map(([hourStart, burnAmount]) => ({ hourStart, burnAmount }))
    .sort((a, b) => a.hourStart - b.hourStart);
}
```

### Computing Burn Deltas from Snapshots

```typescript
function computeBurnDeltas(snapshots: Array<{timestamp: bigint, cycles: bigint}>): BurnDelta[] {
  const deltas: BurnDelta[] = [];

  // Sort oldest to newest
  const sorted = [...snapshots].sort((a, b) =>
    Number(a.timestamp - b.timestamp)
  );

  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1];
    const curr = sorted[i];

    const burnAmount = prev.cycles - curr.cycles;  // Positive if burning

    if (burnAmount > 0n) {
      deltas.push({
        startTime: Number(prev.timestamp / 1_000_000n),  // ns to ms
        endTime: Number(curr.timestamp / 1_000_000n),
        burnAmount,
      });
    } else if (burnAmount < 0n) {
      // Balance increased - track as negative burn for display
      deltas.push({
        startTime: Number(prev.timestamp / 1_000_000n),
        endTime: Number(curr.timestamp / 1_000_000n),
        burnAmount,  // Negative value
      });
    }
  }

  return deltas;
}
```

### Chart Rendering

```typescript
function createBurnChart(container: HTMLElement, snapshots: Snapshot[]) {
  const chart = createChart(container, {
    width: container.clientWidth,
    height: 300,
    layout: {
      background: { color: "#1a1a2e" },
      textColor: "#d1d5db",
    },
    grid: {
      vertLines: { color: "#2d2d44" },
      horzLines: { color: "#2d2d44" },
    },
    timeScale: {
      timeVisible: true,
      secondsVisible: false,
    },
  });

  // Layer 1: Interpolated hourly bars (prominent)
  const barSeries = chart.addHistogramSeries({
    color: "#00d395",
    priceFormat: {
      type: "custom",
      formatter: (v) => formatCycles(v),
    },
  });

  const deltas = computeBurnDeltas(snapshots);
  const hourlyData = interpolateToHourlyBuckets(deltas);

  barSeries.setData(hourlyData.map(h => ({
    time: h.hourStart / 1000,  // ms to seconds for chart
    value: h.burnAmount / 1e12,  // to trillions
    color: h.burnAmount >= 0 ? "#00d395" : "#3b82f6",  // Green for burn, blue for gain
  })));

  // Layer 2: Raw data points (subtle markers)
  const markerSeries = chart.addLineSeries({
    color: "transparent",  // No line, just markers
    lastValueVisible: false,
    priceLineVisible: false,
  });

  // Place markers at midpoint of each measurement
  const markerData = deltas.map(d => ({
    time: (d.startTime + d.endTime) / 2000,  // midpoint, ms to seconds
    value: Number(d.burnAmount) / 1e12 / ((d.endTime - d.startTime) / 3600000),  // rate per hour
  }));

  markerSeries.setData(markerData);

  // Add circle markers for raw data points
  const markers = deltas.map(d => ({
    time: (d.startTime + d.endTime) / 2000,
    position: 'inBar',
    color: '#ffffff40',  // Semi-transparent white
    shape: 'circle',
    size: 1,
  }));
  markerSeries.setMarkers(markers);

  chart.timeScale().fitContent();
  return chart;
}
```

### Chart Controls

```
Range: [1D] [3D] [7D]
```

That's it. No interval selector needed — we always show hourly buckets with the raw points overlaid.

### Hover Tooltip

When hovering over a bar:
```
┌─────────────────────────────┐
│ Jan 5, 2026 9:00-10:00      │
│ Interpolated: 2.1T burned   │
│ ─────────────────────────── │
│ Raw measurements:           │
│  • 8:45-10:15: 3T (90 min)  │
└─────────────────────────────┘
```

Shows both the interpolated value AND the raw measurement(s) that contributed to it.

---

## Stats Panel Updates (Modal)

### Current
```
Current Balance    500T
1h Burn            5T
24h Burn           120T
7d Burn            800T
```

### Proposed
```
Current Balance         500.2T
─────────────────────────────────
Recent (52 min)         5.1T burned
~24 hours (23.5h)       120.4T burned
~7 days (6.9d)          802.1T burned
─────────────────────────────────
Avg burn rate           ~115T/day
```

The "Avg burn rate" is computed from the longest available window (7d) for stability.

---

## Open Questions

1. **Chart toggle default**: Start with Balance view or Burn Rate view?

2. **Burn rate normalization**: Show as T/day or T/hour? (T/day is more intuitive for daily comparisons)

3. **Variable-width bars**: lightweight-charts may not support this natively. Alternative: fixed-width bars but show actual time in tooltip?

4. **Negative burn styling**: In burn chart, negative bars (balance increased) — show below zero line in different color?

5. **Data point markers**: Show dots at each actual snapshot? Or just the line?
