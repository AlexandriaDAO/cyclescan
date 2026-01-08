<script>
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import { getCanisterDetail, getRawSnapshots } from "$lib/data";
  import { linearRegression } from "$lib/regression";
  import { createChart, HistogramSeries, LineSeries } from "lightweight-charts";

  export let canisterId;
  export let onClose;

  const dispatch = createEventDispatcher();

  let data = null;
  let loading = true;
  let error = null;
  let timeRange = "7d";
  let chartContainer;
  let chart = null;

  const TRILLION = 1_000_000_000_000n;
  const BILLION = 1_000_000_000n;
  const MILLION = 1_000_000n;
  const HOUR_MS = 3600000;
  const DAY_MS = 86400000;

  const TIME_RANGES = {
    "1d": DAY_MS,
    "3d": 3 * DAY_MS,
    "7d": 7 * DAY_MS,
  };

  function formatCycles(value) {
    if (value === null || value === undefined) return "-";
    const n = typeof value === 'bigint' ? value : BigInt(value);
    const absN = n < 0n ? -n : n;
    const sign = n < 0n ? "-" : "";

    if (absN >= TRILLION) {
      return sign + (Number(absN / BILLION) / 1000).toFixed(2) + "T";
    } else if (absN >= BILLION) {
      return sign + (Number(absN / MILLION) / 1000).toFixed(2) + "B";
    } else if (absN >= MILLION) {
      return sign + (Number(absN) / 1_000_000).toFixed(2) + "M";
    } else {
      return sign + Number(absN).toLocaleString();
    }
  }

  // Format rate: takes cycles/hour, displays as cycles/day
  function formatRate(ratePerHour) {
    if (ratePerHour === null || ratePerHour === undefined) return "-";
    const perDay = ratePerHour * 24n;

    const isNegative = perDay < 0n;
    const absPerDay = isNegative ? -perDay : perDay;

    let formatted;
    if (absPerDay >= TRILLION) {
      formatted = (Number(absPerDay / BILLION) / 1000).toFixed(2) + "T";
    } else if (absPerDay >= BILLION) {
      formatted = (Number(absPerDay / MILLION) / 1000).toFixed(2) + "B";
    } else if (absPerDay >= MILLION) {
      formatted = (Number(absPerDay) / 1_000_000).toFixed(1) + "M";
    } else {
      formatted = Number(absPerDay).toLocaleString();
    }

    return isNegative ? `+${formatted}` : formatted;
  }

  function formatBurnValue(value) {
    if (value >= 1000) {
      return `${(value / 1000).toFixed(1)}Q`;
    } else if (value >= 1) {
      return `${value.toFixed(2)}T`;
    } else if (value >= 0.001) {
      return `${(value * 1000).toFixed(1)}B`;
    } else {
      return `${(value * 1000000).toFixed(0)}M`;
    }
  }

  function formatTimeDelta(hours) {
    if (hours === null || hours === undefined || hours === 0) return '';
    if (hours < 1) {
      return `${Math.round(hours * 60)} min`;
    } else if (hours < 24) {
      return `${hours.toFixed(1)}h`;
    } else {
      return `${(hours / 24).toFixed(1)}d`;
    }
  }

  function dashboardUrl(id) {
    return `https://dashboard.internetcomputer.org/canister/${id}`;
  }

  async function copyToClipboard(text) {
    try {
      await navigator.clipboard.writeText(text);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }

  function handleKeydown(event) {
    if (event.key === "Escape") {
      onClose();
    }
  }

  function handleOverlayClick(event) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  async function fetchHistory() {
    try {
      loading = true;
      error = null;
      const result = await getCanisterDetail(canisterId);

      if (!result) {
        error = "Canister not found";
        return;
      }

      data = result;
    } catch (e) {
      error = e.message || "Failed to load canister history";
    } finally {
      loading = false;
    }
  }

  // Compute burn deltas between consecutive snapshots
  function computeBurnDeltas(snapshots) {
    if (!snapshots || snapshots.length < 2) return [];

    const sorted = [...snapshots].sort((a, b) =>
      Number(a.timestamp - b.timestamp)
    );

    const deltas = [];
    for (let i = 1; i < sorted.length; i++) {
      const prev = sorted[i - 1];
      const curr = sorted[i];

      const startTimeMs = Number(prev.timestamp / 1_000_000n);
      const endTimeMs = Number(curr.timestamp / 1_000_000n);
      const burnAmount = prev.cycles - curr.cycles;

      deltas.push({
        startTime: startTimeMs,
        endTime: endTimeMs,
        burnAmount: Number(burnAmount),
      });
    }

    return deltas;
  }

  // Interpolate burn deltas into hourly buckets
  function interpolateToHourlyBuckets(deltas) {
    if (deltas.length === 0) return [];

    const hourlyBurns = new Map();

    for (const delta of deltas) {
      const durationMs = delta.endTime - delta.startTime;
      if (durationMs <= 0) continue;

      const burnPerMs = delta.burnAmount / durationMs;

      const startHour = Math.floor(delta.startTime / HOUR_MS) * HOUR_MS;
      const endHour = Math.floor(delta.endTime / HOUR_MS) * HOUR_MS;

      for (let hour = startHour; hour <= endHour; hour += HOUR_MS) {
        const hourEnd = hour + HOUR_MS;

        const overlapStart = Math.max(delta.startTime, hour);
        const overlapEnd = Math.min(delta.endTime, hourEnd);
        const overlapMs = Math.max(0, overlapEnd - overlapStart);

        if (overlapMs > 0) {
          const burnInThisHour = burnPerMs * overlapMs;
          hourlyBurns.set(hour, (hourlyBurns.get(hour) || 0) + burnInThisHour);
        }
      }
    }

    return Array.from(hourlyBurns.entries())
      .map(([hourStart, burnAmount]) => ({ hourStart, burnAmount }))
      .sort((a, b) => a.hourStart - b.hourStart);
  }

  function getFilteredSnapshots() {
    if (!data || !data.snapshots) return [];

    const now = Date.now();
    const rangeMs = TIME_RANGES[timeRange];
    const cutoff = now - rangeMs;

    return data.snapshots.filter(s => {
      const tsMs = Number(s.timestamp / 1_000_000n);
      return tsMs >= cutoff;
    });
  }

  function createChartInstance() {
    if (!chartContainer || !data) return;

    if (chart) {
      chart.remove();
      chart = null;
    }

    const filteredSnapshots = getFilteredSnapshots();
    const deltas = computeBurnDeltas(filteredSnapshots);
    const hourlyData = interpolateToHourlyBuckets(deltas);

    if (hourlyData.length === 0) return;

    chart = createChart(chartContainer, {
      width: chartContainer.clientWidth,
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
        borderColor: "#2d2d44",
      },
      rightPriceScale: {
        borderColor: "#2d2d44",
      },
    });

    // Layer 1: Interpolated hourly bars
    const barSeries = chart.addSeries(HistogramSeries, {
      priceFormat: {
        type: "custom",
        formatter: (price) => formatBurnValue(price),
      },
    });

    const chartData = hourlyData.map(h => ({
      time: h.hourStart / 1000,
      value: h.burnAmount / 1e12,
      color: h.burnAmount >= 0 ? "#00d395" : "#3b82f6",
    }));

    barSeries.setData(chartData);

    // Layer 2: Add trend line using regression
    addTrendLine(chart, filteredSnapshots);

    chart.timeScale().fitContent();
  }

  // Add regression trend line to the chart
  function addTrendLine(chartInstance, snapshots) {
    if (!snapshots || snapshots.length < 2) return;

    // Convert snapshots to points for regression
    const points = snapshots.map(s => ({
      t: Number(s.timestamp / 1_000_000n),  // ms
      v: Number(s.cycles),
    }));

    const result = linearRegression(points);
    if (!result) return;

    // Create line series for trend
    const lineSeries = chartInstance.addSeries(LineSeries, {
      color: 'rgba(249, 115, 22, 0.7)',  // Orange, semi-transparent
      lineWidth: 2,
      lineStyle: 2,  // Dashed
      lastValueVisible: false,
      priceLineVisible: false,
      crosshairMarkerVisible: false,
    });

    // Calculate trend line from regression
    const sortedPoints = [...points].sort((a, b) => a.t - b.t);
    const firstTime = sortedPoints[0].t;
    const lastTime = sortedPoints[sortedPoints.length - 1].t;

    // Calculate balance values from regression line
    const firstValue = result.slope * firstTime + result.intercept;
    const lastValue = result.slope * lastTime + result.intercept;

    // Convert to burn rate for display on the burn chart
    // The trend represents the rate of change
    const burnPerMs = -result.slope;  // Negative slope = positive burn
    const avgHourlyBurn = burnPerMs * HOUR_MS / 1e12;  // Convert to T per hour

    // Create a horizontal reference line at the average burn rate
    const trendData = [
      {
        time: Math.floor(firstTime / 1000),
        value: avgHourlyBurn,
      },
      {
        time: Math.floor(lastTime / 1000),
        value: avgHourlyBurn,
      },
    ];

    lineSeries.setData(trendData);
  }

  function handleResize() {
    if (chart && chartContainer) {
      chart.applyOptions({ width: chartContainer.clientWidth });
    }
  }

  function setTimeRange(range) {
    timeRange = range;
    createChartInstance();
  }

  onMount(async () => {
    await fetchHistory();
    window.addEventListener("resize", handleResize);
  });

  onDestroy(() => {
    window.removeEventListener("resize", handleResize);
    if (chart) {
      chart.remove();
    }
  });

  $: if (data && chartContainer) {
    createChartInstance();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="modal-overlay" on:click={handleOverlayClick}>
  <div class="modal-content" on:click|stopPropagation>
    <button class="close-btn" on:click|stopPropagation={() => onClose()}>&times;</button>

    {#if loading}
      <div class="modal-loading">Loading canister history...</div>
    {:else if error}
      <div class="modal-error">Error: {error}</div>
    {:else if data}
      <div class="modal-header">
        <h2>{data.project?.[0] || "Unknown Project"}</h2>
        <div class="canister-id-row">
          <p class="canister-id-display">{canisterId}</p>
          <button class="copy-btn" on:click={() => copyToClipboard(canisterId)} title="Copy canister ID">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
          </button>
        </div>
      </div>

      <div class="chart-header">
        <span class="chart-title">Hourly Burn (interpolated from snapshots)</span>
      </div>
      <div class="chart-container" bind:this={chartContainer}></div>

      <div class="chart-controls">
        <div class="time-range-selector">
          <span class="control-label">Range</span>
          <div class="range-buttons">
            <button
              class="range-btn"
              class:active={timeRange === "1d"}
              on:click={() => setTimeRange("1d")}
            >1D</button>
            <button
              class="range-btn"
              class:active={timeRange === "3d"}
              on:click={() => setTimeRange("3d")}
            >3D</button>
            <button
              class="range-btn"
              class:active={timeRange === "7d"}
              on:click={() => setTimeRange("7d")}
            >7D</button>
          </div>
        </div>
      </div>

      <div class="stats-panel">
        <div class="stat-row">
          <span class="stat-label">Current Balance</span>
          <span class="stat-value">{formatCycles(data.current_balance)}</span>
        </div>
        <div class="stat-divider"></div>

        <!-- Regression-based burn rates -->
        <div class="stat-row">
          <span class="stat-label">
            Recent Rate
            {#if data.recent_rate?.actualHours}
              <span class="time-delta">({formatTimeDelta(data.recent_rate.actualHours)})</span>
            {/if}
          </span>
          <span class="stat-value" class:gaining={data.recent_rate?.rate < 0n}>
            {#if data.recent_rate}
              {formatRate(data.recent_rate.rate)}/day
              <span class="rate-meta">
                ({data.recent_rate.dataPoints} pts, R²={data.recent_rate.confidence.toFixed(2)})
              </span>
              {#if data.recent_rate.unreliable}
                <span class="topup-indicator" title="Rate may be inaccurate due to top-ups">~</span>
              {/if}
            {:else}
              -
            {/if}
          </span>
        </div>

        <div class="stat-row">
          <span class="stat-label">
            Short-term Rate
            {#if data.short_term_rate?.actualHours}
              <span class="time-delta">({formatTimeDelta(data.short_term_rate.actualHours)})</span>
            {/if}
          </span>
          <span class="stat-value" class:gaining={data.short_term_rate?.rate < 0n}>
            {#if data.short_term_rate}
              {formatRate(data.short_term_rate.rate)}/day
              <span class="rate-meta">
                ({data.short_term_rate.dataPoints} pts, R²={data.short_term_rate.confidence.toFixed(2)})
              </span>
              {#if data.short_term_rate.unreliable}
                <span class="topup-indicator" title="Rate may be inaccurate due to top-ups">~</span>
              {/if}
            {:else}
              -
            {/if}
          </span>
        </div>

        <div class="stat-row">
          <span class="stat-label">
            Long-term Rate
            {#if data.long_term_rate?.actualHours}
              <span class="time-delta">({formatTimeDelta(data.long_term_rate.actualHours)})</span>
            {/if}
          </span>
          <span class="stat-value" class:gaining={data.long_term_rate?.rate < 0n}>
            {#if data.long_term_rate}
              {formatRate(data.long_term_rate.rate)}/day
              <span class="rate-meta">
                ({data.long_term_rate.dataPoints} pts, R²={data.long_term_rate.confidence.toFixed(2)})
              </span>
              {#if data.long_term_rate.unreliable}
                <span class="topup-indicator" title="Rate may be inaccurate due to top-ups">~</span>
              {/if}
            {:else}
              -
            {/if}
          </span>
        </div>
      </div>

      <div class="external-links">
        <a href={dashboardUrl(canisterId)} target="_blank" rel="noopener noreferrer">
          View on IC Dashboard &rarr;
        </a>
      </div>
    {/if}
  </div>
</div>

<style>
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

  .modal-header {
    margin-bottom: 24px;
  }

  .modal-header h2 {
    color: #fff;
    font-size: 24px;
    margin: 0 0 8px 0;
  }

  .canister-id-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .canister-id-display {
    color: #9ca3af;
    font-family: monospace;
    font-size: 14px;
    margin: 0;
  }

  .copy-btn {
    background: transparent;
    border: none;
    color: #9ca3af;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .copy-btn:hover {
    background: #2d2d44;
    color: #00d395;
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
    line-height: 1;
    padding: 0;
    width: 32px;
    height: 32px;
  }

  .close-btn:hover {
    color: #fff;
  }

  .chart-header {
    margin-bottom: 8px;
  }

  .chart-title {
    font-size: 12px;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .chart-container {
    background: #1a1a2e;
    border-radius: 8px;
    margin-bottom: 16px;
    min-height: 300px;
  }

  .chart-controls {
    display: flex;
    gap: 24px;
    margin-bottom: 24px;
    align-items: center;
    flex-wrap: wrap;
  }

  .time-range-selector {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .control-label {
    color: #6b7280;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .range-buttons {
    display: flex;
    background: #2d2d44;
    border-radius: 6px;
    padding: 2px;
  }

  .range-btn {
    background: transparent;
    border: none;
    color: #9ca3af;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .range-btn:hover {
    color: #fff;
  }

  .range-btn.active {
    background: #00d395;
    color: #000;
  }

  .stats-panel {
    background: #2d2d44;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 16px;
  }

  .stat-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 8px 0;
    border-bottom: 1px solid #3d3d54;
    gap: 16px;
  }

  .stat-row:last-child {
    border-bottom: none;
  }

  .stat-divider {
    border-bottom: 2px solid #3d3d54;
    margin: 4px 0;
  }

  .stat-label {
    color: #9ca3af;
    white-space: nowrap;
  }

  .stat-label .time-delta {
    color: #f97316;
    font-size: 12px;
  }

  .stat-value {
    color: #fff;
    font-family: monospace;
    text-align: right;
  }

  .stat-value.gaining {
    color: #3b82f6;
  }

  .rate-meta {
    font-size: 11px;
    color: #6b7280;
    margin-left: 4px;
    font-family: sans-serif;
  }

  .topup-indicator {
    color: #f97316;
    margin-left: 4px;
    font-weight: 600;
  }

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

  .modal-loading,
  .modal-error {
    text-align: center;
    padding: 60px 20px;
    color: #9ca3af;
  }

  .modal-error {
    color: #f85149;
  }
</style>
