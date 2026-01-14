<script>
  import { onMount, onDestroy } from "svelte";
  import { getProjectDetail, getProjectChartIntervals } from "$lib/data";
  import { createChart, HistogramSeries, LineSeries } from "lightweight-charts";

  export let projectName;
  export let onClose;
  export let excludeTransferCanisters = false;

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

  async function fetchData() {
    try {
      loading = true;
      error = null;
      const result = await getProjectDetail(projectName);

      if (!result) {
        error = "Project not found";
        return;
      }

      data = result;
    } catch (e) {
      error = e.message || "Failed to load project data";
    } finally {
      loading = false;
    }
  }

  function createChartInstance() {
    if (!chartContainer || !projectName) return;

    if (chart) {
      chart.remove();
      chart = null;
    }

    // Get aggregated intervals for the project
    const rangeMs = TIME_RANGES[timeRange];
    const intervals = getProjectChartIntervals(projectName, rangeMs, excludeTransferCanisters);

    if (intervals.length === 0) return;

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

    // Create chart data from intervals
    const chartData = [];

    for (const interval of intervals) {
      const midTime = Math.floor((interval.startTime + interval.endTime) / 2000); // seconds for chart
      const durationHours = interval.duration / HOUR_MS;

      if (interval.isTopUp) {
        // For top-up intervals, show inferred burn (orange)
        const inferredBurnPerHour = interval.inferredBurn / interval.duration * HOUR_MS;
        if (inferredBurnPerHour > 0) {
          chartData.push({
            time: midTime,
            value: inferredBurnPerHour / 1e12, // Convert to T
            color: "#f97316", // Orange for inferred
            isInferred: true,
          });
        }

        // Show top-up as negative bar
        const topUpPerHour = interval.topUpAmount / interval.duration * HOUR_MS;
        if (topUpPerHour > 0) {
          chartData.push({
            time: midTime + 1, // Offset by 1 second to avoid overlap
            value: -topUpPerHour / 1e12, // Negative for top-up
            color: "#f85149", // Red for top-up
            isTopUp: true,
          });
        }
      } else {
        // Regular burn interval - green bar
        const burnPerHour = interval.actualBurn / interval.duration * HOUR_MS;
        chartData.push({
          time: midTime,
          value: burnPerHour / 1e12, // Convert to T
          color: "#00d395", // Green for actual burn
          isActual: true,
        });
      }
    }

    // Sort by time
    chartData.sort((a, b) => a.time - b.time);

    if (chartData.length === 0) return;

    // Create histogram series
    const barSeries = chart.addSeries(HistogramSeries, {
      priceFormat: {
        type: "custom",
        formatter: (price) => formatBurnValue(Math.abs(price)),
      },
    });

    barSeries.setData(chartData);

    // Add average burn rate line
    addAverageLine(chart, intervals);

    chart.timeScale().fitContent();
  }

  // Add horizontal line showing average burn rate
  function addAverageLine(chartInstance, intervals) {
    if (!intervals || intervals.length === 0) return;

    // Calculate average from actual burn intervals only
    const burnIntervals = intervals.filter(i => !i.isTopUp);
    if (burnIntervals.length === 0) return;

    const totalBurn = burnIntervals.reduce((s, i) => s + i.actualBurn, 0);
    const totalDuration = burnIntervals.reduce((s, i) => s + i.duration, 0);

    if (totalDuration <= 0) return;

    const avgBurnPerMs = totalBurn / totalDuration;
    const avgBurnPerHourT = avgBurnPerMs * HOUR_MS / 1e12;

    // Create line series for average
    const lineSeries = chartInstance.addSeries(LineSeries, {
      color: 'rgba(249, 115, 22, 0.7)',  // Orange, semi-transparent
      lineWidth: 2,
      lineStyle: 2,  // Dashed
      lastValueVisible: false,
      priceLineVisible: false,
      crosshairMarkerVisible: false,
    });

    // Get time range from intervals
    const firstTime = Math.floor(intervals[0].startTime / 1000);
    const lastTime = Math.floor(intervals[intervals.length - 1].endTime / 1000);

    lineSeries.setData([
      { time: firstTime, value: avgBurnPerHourT },
      { time: lastTime, value: avgBurnPerHourT },
    ]);
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
    await fetchData();
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
      <div class="modal-loading">Loading project data...</div>
    {:else if error}
      <div class="modal-error">Error: {error}</div>
    {:else if data}
      <div class="modal-header">
        <h2>{projectName}</h2>
        <p class="project-meta">
          {Number(data.canister_count).toLocaleString()} canister{data.canister_count !== 1n ? 's' : ''}
          {#if data.website?.[0]}
            <a href={data.website[0]} target="_blank" rel="noopener noreferrer" class="website-link">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="2" y1="12" x2="22" y2="12"></line>
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
              </svg>
            </a>
          {/if}
        </p>
      </div>

      <div class="chart-header">
        <span class="chart-title">Aggregate Burn Rate</span>
        <div class="chart-legend">
          <span class="legend-item"><span class="legend-dot actual"></span> Actual</span>
          <span class="legend-item"><span class="legend-dot inferred"></span> Inferred</span>
          <span class="legend-item"><span class="legend-dot topup"></span> Top-up</span>
        </div>
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
          <span class="stat-label">Total Balance</span>
          <span class="stat-value">{formatCycles(data.total_balance)}</span>
        </div>
        <div class="stat-divider"></div>

        <!-- Burn rates -->
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
                ({data.recent_rate.canistersWithData || data.recent_rate.dataPoints} {data.recent_rate.canistersWithData ? 'canisters' : 'pts'}{#if data.recent_rate.topUpsDetected > 0}, {data.recent_rate.topUpsDetected} top-up{data.recent_rate.topUpsDetected > 1 ? 's' : ''}{/if})
              </span>
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
                ({data.short_term_rate.canistersWithData || data.short_term_rate.dataPoints} {data.short_term_rate.canistersWithData ? 'canisters' : 'pts'}{#if data.short_term_rate.topUpsDetected > 0}, {data.short_term_rate.topUpsDetected} top-up{data.short_term_rate.topUpsDetected > 1 ? 's' : ''}{/if})
              </span>
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
                ({data.long_term_rate.canistersWithData || data.long_term_rate.dataPoints} {data.long_term_rate.canistersWithData ? 'canisters' : 'pts'}{#if data.long_term_rate.topUpsDetected > 0}, {data.long_term_rate.topUpsDetected} top-up{data.long_term_rate.topUpsDetected > 1 ? 's' : ''}{/if})
              </span>
            {:else}
              -
            {/if}
          </span>
        </div>
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

  .project-meta {
    color: #9ca3af;
    font-size: 14px;
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .website-link {
    color: #9ca3af;
    display: inline-flex;
    align-items: center;
    transition: color 0.15s ease;
  }

  .website-link:hover {
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
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }

  .chart-title {
    font-size: 12px;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .chart-legend {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: #9ca3af;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .legend-dot {
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }

  .legend-dot.actual {
    background: #00d395;
  }

  .legend-dot.inferred {
    background: #f97316;
  }

  .legend-dot.topup {
    background: #f85149;
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
