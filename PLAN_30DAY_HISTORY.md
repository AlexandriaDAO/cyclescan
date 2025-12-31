# CycleScan: 30-Day History & Canister Detail View

**Date:** 2025-12-31
**Status:** Planning
**Priority:** Medium

---

## Overview

Extend CycleScan to store 30 days of hourly snapshots, implement hybrid burn calculation (actual data when available, extrapolated when not), and add a modal detail view with CoinGecko-style charts when clicking on individual canisters.

---

## Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Chart library | Lightweight Charts (TradingView) | CoinGecko-style appearance, financial data optimized |
| Chart type | Bar charts | Better for discrete hourly data points |
| Detail view | Modal overlay | Simpler implementation, no routing changes |
| Burn calculation | Hybrid | Actual data when available, extrapolate when insufficient |
| Retention period | 30 days | Balances storage cost with useful historical depth |

---

## Storage Analysis

### Current State
```
Canisters tracked:     3,138
Current snapshots:     ~60,000
Current retention:     7 days
```

### Projected State (30 days)
```
Snapshots per canister:  720 (24 hours × 30 days)
Total snapshots:         2,259,360 (3,138 × 720)
Storage per snapshot:    54 bytes (38 key + 16 value)
Total storage:           ~122 MB
ICP stable memory limit: 4 GB
Usage:                   ~3%
```

### Storage Breakdown
| Component | Size | Notes |
|-----------|------|-------|
| SnapshotKey | 38 bytes | 30 (PrincipalKey) + 8 (u64 timestamp) |
| CyclesValue | 16 bytes | u128 |
| Per snapshot | 54 bytes | Key + value |
| 30 days all canisters | 122 MB | Well within limits |

**Verdict:** Storage is not a concern. Could even extend to 90 days (~366 MB) if needed later.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐         ┌─────────────────────────────┐   │
│  │   Leaderboard   │─click──▶│     CanisterDetailModal     │   │
│  │     Table       │         │                             │   │
│  └─────────────────┘         │  ┌───────────────────────┐  │   │
│          │                   │  │  Lightweight Charts   │  │   │
│          │                   │  │  (Bar Chart)          │  │   │
│          ▼                   │  └───────────────────────┘  │   │
│  get_leaderboard()           │                             │   │
│                              │  Stats Panel                │   │
│                              │  Time Range Selector        │   │
│                              └─────────────────────────────┘   │
│                                        │                        │
│                                        ▼                        │
│                              get_canister_history()             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Backend                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │  CANISTERS   │    │  SNAPSHOTS   │    │  Burn Calculator │  │
│  │  StableBTree │    │  StableBTree │    │  (Hybrid Logic)  │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                 │
│  Endpoints:                                                     │
│  ├── get_leaderboard() -> Vec<LeaderboardEntry>                │
│  ├── get_canister_history(principal) -> CanisterHistory        │
│  ├── get_stats() -> Stats                                      │
│  └── take_snapshot() -> SnapshotResult                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Backend Implementation

### File: `src/cyclescan_backend/src/lib.rs`

#### Change 1: Update Constants

```rust
// BEFORE
const SEVEN_DAYS_NANOS: u64 = 7 * NANOS_PER_DAY;

// AFTER
const SEVEN_DAYS_NANOS: u64 = 7 * NANOS_PER_DAY;
const THIRTY_DAYS_NANOS: u64 = 30 * NANOS_PER_DAY;
const RETENTION_PERIOD: u64 = THIRTY_DAYS_NANOS;
```

#### Change 2: Update Pruning in `take_snapshot()`

```rust
// BEFORE (line ~725)
let cutoff = timestamp.saturating_sub(SEVEN_DAYS_NANOS);

// AFTER
let cutoff = timestamp.saturating_sub(RETENTION_PERIOD);
```

#### Change 3: Hybrid Burn Calculation

Replace the current `calculate_burn` function:

```rust
/// Calculate burn for a time window using hybrid approach:
/// - If we have actual data spanning the window, use it
/// - Otherwise, extrapolate from the last 2 snapshots
fn calculate_burn(canister: &PrincipalKey, window_nanos: u64, now: u64) -> Option<u128> {
    SNAPSHOTS.with(|s| {
        let map = s.borrow();

        // Get all snapshots for this canister
        let start_key = SnapshotKey {
            canister: canister.clone(),
            timestamp: 0,
        };
        let end_key = SnapshotKey {
            canister: canister.clone(),
            timestamp: u64::MAX,
        };

        let snapshots: Vec<_> = map.range(start_key..=end_key).collect();

        if snapshots.len() < 2 {
            return None;
        }

        // Get the newest snapshot
        let (newest_key, newest_val) = snapshots.last().unwrap();
        let newest_cycles = newest_val.0;
        let newest_time = newest_key.timestamp;

        // Calculate cutoff for the requested window
        let cutoff = now.saturating_sub(window_nanos);

        // Try to find a snapshot at or before the cutoff (actual data available)
        let older_snapshot = snapshots.iter()
            .filter(|(key, _)| key.timestamp <= cutoff)
            .last();

        match older_snapshot {
            Some((older_key, older_val)) => {
                // ACTUAL DATA: We have snapshots spanning the full window
                let older_cycles = older_val.0;

                // Handle top-ups (cycles increased)
                if newest_cycles >= older_cycles {
                    Some(0)
                } else {
                    Some(older_cycles - newest_cycles)
                }
            }
            None => {
                // EXTRAPOLATE: Not enough historical data, use last 2 snapshots
                let len = snapshots.len();
                let (older_key, older_val) = &snapshots[len - 2];
                let (newer_key, newer_val) = &snapshots[len - 1];

                let older_cycles = older_val.0;
                let newer_cycles = newer_val.0;
                let time_elapsed = newer_key.timestamp.saturating_sub(older_key.timestamp);

                if time_elapsed == 0 {
                    return Some(0);
                }

                // Handle top-ups
                if newer_cycles >= older_cycles {
                    return Some(0);
                }

                let actual_burn = older_cycles - newer_cycles;

                // Extrapolate to requested window
                let burn_per_nano = actual_burn as f64 / time_elapsed as f64;
                let projected_burn = burn_per_nano * window_nanos as f64;

                Some(projected_burn as u128)
            }
        }
    })
}
```

#### Change 4: Add New Types

```rust
/// A single snapshot point for history
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SnapshotPoint {
    pub timestamp: u64,
    pub cycles: u128,
}

/// Full history for a canister (for detail view)
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CanisterHistory {
    pub canister_id: Principal,
    pub project: Option<String>,
    pub current_balance: u128,
    pub snapshots: Vec<SnapshotPoint>,
    pub burn_1h: Option<u128>,
    pub burn_24h: Option<u128>,
    pub burn_7d: Option<u128>,
    pub burn_30d: Option<u128>,
    pub is_24h_actual: bool,  // true if 24h burn is from actual data
    pub is_7d_actual: bool,   // true if 7d burn is from actual data
    pub is_30d_actual: bool,  // true if 30d burn is from actual data
}
```

#### Change 5: Add `get_canister_history` Endpoint

```rust
/// Get full history for a specific canister (for detail modal)
#[ic_cdk::query]
fn get_canister_history(canister_id: Principal) -> Option<CanisterHistory> {
    let key = PrincipalKey::new(canister_id);
    let now = now_nanos();

    // Get canister metadata
    let meta = CANISTERS.with(|c| c.borrow().get(&key))?;

    // Get all snapshots for this canister
    let snapshots: Vec<SnapshotPoint> = SNAPSHOTS.with(|s| {
        let map = s.borrow();

        let start_key = SnapshotKey {
            canister: key.clone(),
            timestamp: 0,
        };
        let end_key = SnapshotKey {
            canister: key.clone(),
            timestamp: u64::MAX,
        };

        map.range(start_key..=end_key)
            .map(|(k, v)| SnapshotPoint {
                timestamp: k.timestamp,
                cycles: v.0,
            })
            .collect()
    });

    if snapshots.is_empty() {
        return None;
    }

    let current_balance = snapshots.last().map(|s| s.cycles).unwrap_or(0);

    // Calculate burns and determine if they're actual or extrapolated
    let burn_24h = calculate_burn(&key, NANOS_PER_DAY, now);
    let burn_7d = calculate_burn(&key, SEVEN_DAYS_NANOS, now);
    let burn_30d = calculate_burn(&key, THIRTY_DAYS_NANOS, now);

    // Determine if we have actual data for each window
    let oldest_timestamp = snapshots.first().map(|s| s.timestamp).unwrap_or(now);
    let data_span = now.saturating_sub(oldest_timestamp);

    let is_24h_actual = data_span >= NANOS_PER_DAY;
    let is_7d_actual = data_span >= SEVEN_DAYS_NANOS;
    let is_30d_actual = data_span >= THIRTY_DAYS_NANOS;

    Some(CanisterHistory {
        canister_id,
        project: meta.project_name,
        current_balance,
        snapshots,
        burn_1h: calculate_burn(&key, NANOS_PER_HOUR, now),
        burn_24h,
        burn_7d,
        burn_30d,
        is_24h_actual,
        is_7d_actual,
        is_30d_actual,
    })
}
```

#### Change 6: Update LeaderboardEntry (Optional)

Add indicator for actual vs extrapolated:

```rust
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LeaderboardEntry {
    pub canister_id: Principal,
    pub project: Option<String>,
    pub balance: u128,
    pub burn_1h: Option<u128>,
    pub burn_24h: Option<u128>,
    pub burn_7d: Option<u128>,
    pub is_7d_actual: bool,  // NEW: indicates if 7d is actual or extrapolated
}
```

#### Change 7: Update Candid Interface

File: `src/cyclescan_backend/cyclescan_backend.did`

```candid
type SnapshotPoint = record {
    timestamp : nat64;
    cycles : nat;
};

type CanisterHistory = record {
    canister_id : principal;
    project : opt text;
    current_balance : nat;
    snapshots : vec SnapshotPoint;
    burn_1h : opt nat;
    burn_24h : opt nat;
    burn_7d : opt nat;
    burn_30d : opt nat;
    is_24h_actual : bool;
    is_7d_actual : bool;
    is_30d_actual : bool;
};

service : {
    // Existing
    get_leaderboard : () -> (vec LeaderboardEntry) query;
    get_stats : () -> (Stats) query;
    get_canister_count : () -> (nat64) query;
    take_snapshot : () -> (SnapshotResult);
    import_canisters : (vec CanisterImport) -> (nat64);
    set_project : (principal, opt text) -> ();
    clear_canisters : () -> ();
    clear_snapshots : () -> ();
    start_timer : () -> ();
    stop_timer : () -> ();
    is_timer_running : () -> (bool) query;

    // NEW
    get_canister_history : (principal) -> (opt CanisterHistory) query;
}
```

---

## Frontend Implementation

### Dependencies to Add

```json
{
  "dependencies": {
    "lightweight-charts": "^4.1.0"
  }
}
```

### File Structure

```
src/cyclescan_frontend/src/
├── components/
│   ├── Leaderboard.tsx          # Existing, add onClick handler
│   ├── CanisterDetailModal.tsx  # NEW: Modal with chart
│   ├── CyclesChart.tsx          # NEW: Lightweight Charts wrapper
│   └── TimeRangeSelector.tsx    # NEW: 1d/7d/30d buttons
├── hooks/
│   └── useCanisterHistory.ts    # NEW: Fetch hook
├── utils/
│   └── formatCycles.ts          # Existing cycle formatting
└── App.tsx                      # Add modal state
```

### Component: CanisterDetailModal.tsx

```tsx
import React, { useEffect, useState } from 'react';
import { Principal } from '@dfinity/principal';
import { CyclesChart } from './CyclesChart';
import { TimeRangeSelector } from './TimeRangeSelector';
import { useCanisterHistory } from '../hooks/useCanisterHistory';
import { formatCycles } from '../utils/formatCycles';

interface Props {
  canisterId: string;
  onClose: () => void;
}

type TimeRange = '1d' | '7d' | '30d';

export function CanisterDetailModal({ canisterId, onClose }: Props) {
  const [timeRange, setTimeRange] = useState<TimeRange>('7d');
  const { data, loading, error } = useCanisterHistory(canisterId);

  if (loading) return <ModalSkeleton onClose={onClose} />;
  if (error || !data) return <ModalError onClose={onClose} error={error} />;

  // Filter snapshots based on time range
  const now = Date.now() * 1_000_000; // Convert to nanoseconds
  const rangeNanos = {
    '1d': 86_400_000_000_000,
    '7d': 7 * 86_400_000_000_000,
    '30d': 30 * 86_400_000_000_000,
  }[timeRange];

  const cutoff = now - rangeNanos;
  const filteredSnapshots = data.snapshots.filter(s => s.timestamp >= cutoff);

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        {/* Header */}
        <div className="modal-header">
          <button className="close-btn" onClick={onClose}>×</button>
          <h2>{data.project || 'Unknown Project'}</h2>
          <p className="canister-id">{canisterId}</p>
        </div>

        {/* Chart */}
        <div className="chart-container">
          <CyclesChart
            data={filteredSnapshots}
            timeRange={timeRange}
          />
        </div>

        {/* Time Range Selector */}
        <TimeRangeSelector
          value={timeRange}
          onChange={setTimeRange}
          has24hData={data.is_24h_actual}
          has7dData={data.is_7d_actual}
          has30dData={data.is_30d_actual}
        />

        {/* Stats Panel */}
        <div className="stats-panel">
          <StatRow
            label="Current Balance"
            value={formatCycles(data.current_balance)}
          />
          <StatRow
            label="1h Burn"
            value={formatCycles(data.burn_1h)}
            extrapolated={true}
          />
          <StatRow
            label="24h Burn"
            value={formatCycles(data.burn_24h)}
            extrapolated={!data.is_24h_actual}
          />
          <StatRow
            label="7d Burn"
            value={formatCycles(data.burn_7d)}
            extrapolated={!data.is_7d_actual}
          />
          <StatRow
            label="30d Burn"
            value={formatCycles(data.burn_30d)}
            extrapolated={!data.is_30d_actual}
          />
        </div>

        {/* Links */}
        <div className="external-links">
          <a
            href={`https://dashboard.internetcomputer.org/canister/${canisterId}`}
            target="_blank"
            rel="noopener noreferrer"
          >
            View on IC Dashboard →
          </a>
        </div>
      </div>
    </div>
  );
}

function StatRow({ label, value, extrapolated }: {
  label: string;
  value: string;
  extrapolated?: boolean;
}) {
  return (
    <div className="stat-row">
      <span className="stat-label">{label}</span>
      <span className="stat-value">
        {value}
        {extrapolated && <span className="extrapolated-badge">~</span>}
      </span>
    </div>
  );
}
```

### Component: CyclesChart.tsx

```tsx
import React, { useEffect, useRef } from 'react';
import { createChart, IChartApi, HistogramSeries } from 'lightweight-charts';

interface SnapshotPoint {
  timestamp: number;  // nanoseconds
  cycles: bigint;
}

interface Props {
  data: SnapshotPoint[];
  timeRange: '1d' | '7d' | '30d';
}

export function CyclesChart({ data, timeRange }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Create chart with CoinGecko-style theme
    const chart = createChart(containerRef.current, {
      width: containerRef.current.clientWidth,
      height: 300,
      layout: {
        background: { color: '#1a1a2e' },
        textColor: '#d1d5db',
      },
      grid: {
        vertLines: { color: '#2d2d44' },
        horzLines: { color: '#2d2d44' },
      },
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        borderColor: '#2d2d44',
      },
      rightPriceScale: {
        borderColor: '#2d2d44',
      },
    });

    chartRef.current = chart;

    // Add bar series (histogram) for cycles balance
    const barSeries = chart.addHistogramSeries({
      color: '#00d395',  // CoinGecko green
      priceFormat: {
        type: 'custom',
        formatter: (price: number) => formatTrillions(price),
      },
    });

    // Convert data for chart
    const chartData = data.map(point => ({
      time: Math.floor(Number(point.timestamp) / 1_000_000_000) as any, // seconds
      value: Number(point.cycles) / 1e12, // Convert to trillions for readability
      color: '#00d395',
    }));

    barSeries.setData(chartData);

    // Fit content
    chart.timeScale().fitContent();

    // Handle resize
    const handleResize = () => {
      if (containerRef.current) {
        chart.applyOptions({ width: containerRef.current.clientWidth });
      }
    };
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      chart.remove();
    };
  }, [data, timeRange]);

  return <div ref={containerRef} className="cycles-chart" />;
}

function formatTrillions(value: number): string {
  if (value >= 1000) {
    return `${(value / 1000).toFixed(1)}Q`;  // Quadrillions
  }
  return `${value.toFixed(1)}T`;  // Trillions
}
```

### Component: TimeRangeSelector.tsx

```tsx
import React from 'react';

interface Props {
  value: '1d' | '7d' | '30d';
  onChange: (range: '1d' | '7d' | '30d') => void;
  has24hData: boolean;
  has7dData: boolean;
  has30dData: boolean;
}

export function TimeRangeSelector({
  value,
  onChange,
  has24hData,
  has7dData,
  has30dData,
}: Props) {
  const options = [
    { key: '1d', label: '1D', hasData: has24hData },
    { key: '7d', label: '7D', hasData: has7dData },
    { key: '30d', label: '30D', hasData: has30dData },
  ] as const;

  return (
    <div className="time-range-selector">
      {options.map(opt => (
        <button
          key={opt.key}
          className={`range-btn ${value === opt.key ? 'active' : ''}`}
          onClick={() => onChange(opt.key)}
        >
          {opt.label}
          {!opt.hasData && <span className="no-data-indicator">*</span>}
        </button>
      ))}
      <span className="data-note">* extrapolated data</span>
    </div>
  );
}
```

### Hook: useCanisterHistory.ts

```tsx
import { useState, useEffect } from 'react';
import { Principal } from '@dfinity/principal';
import { useActor } from './useActor';  // Your existing actor hook

interface CanisterHistory {
  canister_id: Principal;
  project: string | null;
  current_balance: bigint;
  snapshots: Array<{ timestamp: bigint; cycles: bigint }>;
  burn_1h: bigint | null;
  burn_24h: bigint | null;
  burn_7d: bigint | null;
  burn_30d: bigint | null;
  is_24h_actual: boolean;
  is_7d_actual: boolean;
  is_30d_actual: boolean;
}

export function useCanisterHistory(canisterId: string) {
  const [data, setData] = useState<CanisterHistory | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const actor = useActor();

  useEffect(() => {
    async function fetch() {
      try {
        setLoading(true);
        const principal = Principal.fromText(canisterId);
        const result = await actor.get_canister_history(principal);

        if (result.length === 0 || !result[0]) {
          setError(new Error('Canister not found'));
          return;
        }

        setData(result[0]);
      } catch (e) {
        setError(e as Error);
      } finally {
        setLoading(false);
      }
    }

    fetch();
  }, [canisterId, actor]);

  return { data, loading, error };
}
```

### Styles: modal.css

```css
/* Modal Overlay */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

/* Modal Content */
.modal-content {
  background: #1a1a2e;
  border-radius: 16px;
  width: 90%;
  max-width: 800px;
  max-height: 90vh;
  overflow-y: auto;
  padding: 24px;
  position: relative;
}

/* Header */
.modal-header {
  margin-bottom: 24px;
}

.modal-header h2 {
  color: #fff;
  font-size: 24px;
  margin: 0 0 8px 0;
}

.canister-id {
  color: #9ca3af;
  font-family: monospace;
  font-size: 14px;
}

.close-btn {
  position: absolute;
  top: 16px;
  right: 16px;
  background: none;
  border: none;
  color: #9ca3af;
  font-size: 28px;
  cursor: pointer;
}

.close-btn:hover {
  color: #fff;
}

/* Chart */
.chart-container {
  background: #1a1a2e;
  border-radius: 8px;
  margin-bottom: 16px;
}

/* Time Range Selector */
.time-range-selector {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  align-items: center;
}

.range-btn {
  background: #2d2d44;
  border: none;
  color: #9ca3af;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}

.range-btn:hover {
  background: #3d3d54;
}

.range-btn.active {
  background: #00d395;
  color: #000;
}

.no-data-indicator {
  margin-left: 4px;
  color: #f59e0b;
}

.data-note {
  color: #6b7280;
  font-size: 12px;
  margin-left: auto;
}

/* Stats Panel */
.stats-panel {
  background: #2d2d44;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #3d3d54;
}

.stat-row:last-child {
  border-bottom: none;
}

.stat-label {
  color: #9ca3af;
}

.stat-value {
  color: #fff;
  font-family: monospace;
}

.extrapolated-badge {
  color: #f59e0b;
  margin-left: 4px;
  font-size: 12px;
}

/* External Links */
.external-links {
  text-align: center;
}

.external-links a {
  color: #00d395;
  text-decoration: none;
}

.external-links a:hover {
  text-decoration: underline;
}
```

---

## Modal UI Mockup

```
┌──────────────────────────────────────────────────────────────────┐
│                                                              [×] │
│  ODIN.fun                                                        │
│  ryjl3-tyaaa-aaaaa-aaaba-cai                                    │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                                                            │ │
│  │  ████                                                      │ │
│  │  ████ ████                                                 │ │
│  │  ████ ████ ████                                            │ │
│  │  ████ ████ ████ ████                                       │ │
│  │  ████ ████ ████ ████ ████                                  │ │
│  │  ████ ████ ████ ████ ████ ████                             │ │
│  │  ████ ████ ████ ████ ████ ████ ████ ████ ████ ████        │ │
│  │  ────────────────────────────────────────────────────      │ │
│  │   Dec 24    Dec 26    Dec 28    Dec 30                     │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  [1D]  [7D]  [30D*]                    * extrapolated data      │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Current Balance              177,746.68T                  │ │
│  │  ──────────────────────────────────────────────────────── │ │
│  │  1h Burn                      8.07T ~                      │ │
│  │  ──────────────────────────────────────────────────────── │ │
│  │  24h Burn                     193.74T                      │ │
│  │  ──────────────────────────────────────────────────────── │ │
│  │  7d Burn                      1,356.20T                    │ │
│  │  ──────────────────────────────────────────────────────── │ │
│  │  30d Burn                     5,424.80T ~                  │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│              View on IC Dashboard →                              │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

Legend:
  ~ = extrapolated (not enough historical data)
  ████ = bar representing cycles balance at that hour
```

---

## Implementation Checklist

### Phase 1: Backend (Est. 30 min)

- [ ] Update `RETENTION_PERIOD` constant to 30 days
- [ ] Update pruning logic in `take_snapshot()`
- [ ] Implement hybrid `calculate_burn()` function
- [ ] Add `SnapshotPoint` type
- [ ] Add `CanisterHistory` type
- [ ] Add `get_canister_history()` endpoint
- [ ] Update Candid interface file
- [ ] Add `burn_30d` to `LeaderboardEntry` (optional)
- [ ] Add `is_7d_actual` flag to `LeaderboardEntry` (optional)
- [ ] Run `cargo build --target wasm32-unknown-unknown --release`
- [ ] Deploy to mainnet

### Phase 2: Backend Testing (Est. 15 min)

- [ ] Verify `get_stats()` still works
- [ ] Verify `get_leaderboard()` still works
- [ ] Test `get_canister_history()` with known canister
- [ ] Verify hybrid burn calculation returns different values for actual vs extrapolated
- [ ] Verify pruning only removes >30 day old snapshots

### Phase 3: Frontend - Modal Structure (Est. 1 hour)

- [ ] Install `lightweight-charts` package
- [ ] Create `CanisterDetailModal.tsx` component
- [ ] Create `useCanisterHistory.ts` hook
- [ ] Add modal state to `App.tsx` or `Leaderboard.tsx`
- [ ] Add click handler to leaderboard rows
- [ ] Test modal opens/closes correctly
- [ ] Test data fetches correctly

### Phase 4: Frontend - Chart (Est. 1 hour)

- [ ] Create `CyclesChart.tsx` component
- [ ] Configure Lightweight Charts with dark theme
- [ ] Add bar series for cycles data
- [ ] Format Y-axis labels (T/Q suffixes)
- [ ] Format X-axis with dates
- [ ] Add hover tooltips
- [ ] Handle resize

### Phase 5: Frontend - Polish (Est. 1 hour)

- [ ] Create `TimeRangeSelector.tsx` component
- [ ] Add filtering by time range
- [ ] Add stats panel with burn values
- [ ] Add extrapolated indicator (~)
- [ ] Add external link to IC Dashboard
- [ ] Style modal with CSS
- [ ] Mobile responsiveness
- [ ] Loading skeleton
- [ ] Error state

### Phase 6: Deploy & Verify (Est. 15 min)

- [ ] Build frontend
- [ ] Deploy frontend canister
- [ ] Test on live site
- [ ] Verify chart renders correctly
- [ ] Verify time range switching works
- [ ] Verify extrapolated indicators display correctly

---

## Testing Scenarios

### Backend Tests

| Test | Expected Result |
|------|-----------------|
| `get_canister_history` for canister with 5 snapshots | Returns all 5 snapshots, all burns extrapolated |
| `get_canister_history` for canister with 25 hours of data | `is_24h_actual = true`, `is_7d_actual = false` |
| `get_canister_history` for canister with 8 days of data | `is_24h_actual = true`, `is_7d_actual = true`, `is_30d_actual = false` |
| `get_canister_history` for unknown canister | Returns `None` |
| `calculate_burn` with actual 7d data | Returns actual burn (first - last) |
| `calculate_burn` with only 2 days data for 7d window | Returns extrapolated burn |

### Frontend Tests

| Test | Expected Result |
|------|-----------------|
| Click canister row | Modal opens with chart |
| Click outside modal | Modal closes |
| Click X button | Modal closes |
| Select 1D range | Chart shows last 24 hours |
| Select 30D range with <30d data | Chart shows all data, asterisk on button |
| Hover on chart bar | Tooltip shows exact cycles value |
| View on mobile | Modal is scrollable, chart resizes |

---

## Rollback Plan

If issues arise after deployment:

1. **Backend issues**: Redeploy previous WASM (canister state preserved)
2. **Frontend issues**: Redeploy previous frontend assets
3. **Data issues**: Data is append-only, no destructive changes in this plan

---

## Future Enhancements (Out of Scope)

- [ ] CSV export of historical data
- [ ] Compare multiple canisters on same chart
- [ ] Alert when burn rate exceeds threshold
- [ ] Daily/weekly email reports
- [ ] Public API for third-party integrations
- [ ] Extend to 90 days retention
- [ ] Aggregate hourly data to daily for older periods (compression)

---

## References

- Backend code: `src/cyclescan_backend/src/lib.rs`
- Frontend code: `src/cyclescan_frontend/src/`
- Lightweight Charts docs: https://tradingview.github.io/lightweight-charts/
- CoinGecko charts (reference): https://www.coingecko.com/en/coins/bitcoin
- Current deployment: `vohji-riaaa-aaaac-babxq-cai`
