<script>
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import { backend } from "$lib/canisters";
  import { Principal } from "@dfinity/principal";
  import { createChart, HistogramSeries } from "lightweight-charts";

  export let canisterId;
  export let onClose;

  const dispatch = createEventDispatcher();

  let data = null;
  let loading = true;
  let error = null;
  let timeRange = "7d";
  let chartContainer;
  let chart = null;
  let barSeries = null;

  const TRILLION = 1_000_000_000_000n;
  const BILLION = 1_000_000_000n;
  const MILLION = 1_000_000n;
  const NANOS_PER_DAY = 86_400_000_000_000n;

  const TIME_RANGES = {
    "1d": NANOS_PER_DAY,
    "7d": 7n * NANOS_PER_DAY,
    "30d": 30n * NANOS_PER_DAY,
  };

  function formatCycles(value) {
    if (value === null || value === undefined) return "-";
    const n = BigInt(value);
    if (n >= TRILLION) {
      return (Number(n / BILLION) / 1000).toFixed(2) + "T";
    } else if (n >= BILLION) {
      return (Number(n / MILLION) / 1000).toFixed(2) + "B";
    } else if (n >= MILLION) {
      return (Number(n) / 1_000_000).toFixed(2) + "M";
    } else {
      return Number(n).toLocaleString();
    }
  }

  function formatTrillions(value) {
    if (value >= 1000) {
      return `${(value / 1000).toFixed(1)}Q`;
    }
    return `${value.toFixed(1)}T`;
  }

  function dashboardUrl(id) {
    return `https://dashboard.internetcomputer.org/canister/${id}`;
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
      const principal = Principal.fromText(canisterId);
      const result = await backend.get_canister_history(principal);

      if (result.length === 0 || !result[0]) {
        error = "Canister not found";
        return;
      }

      data = result[0];
    } catch (e) {
      error = e.message || "Failed to load canister history";
    } finally {
      loading = false;
    }
  }

  function getFilteredSnapshots() {
    if (!data || !data.snapshots) return [];

    const now = BigInt(Date.now()) * 1_000_000n; // Convert to nanoseconds
    const rangeNanos = TIME_RANGES[timeRange];
    const cutoff = now - rangeNanos;

    return data.snapshots.filter(s => BigInt(s.timestamp) >= cutoff);
  }

  function createChartInstance() {
    if (!chartContainer || !data) return;

    // Destroy existing chart
    if (chart) {
      chart.remove();
      chart = null;
    }

    const filteredSnapshots = getFilteredSnapshots();
    if (filteredSnapshots.length === 0) return;

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

    barSeries = chart.addSeries(HistogramSeries, {
      color: "#00d395",
      priceFormat: {
        type: "custom",
        formatter: (price) => formatTrillions(price),
      },
    });

    const chartData = filteredSnapshots.map(point => ({
      time: Math.floor(Number(point.timestamp) / 1_000_000_000), // seconds
      value: Number(point.cycles) / 1e12, // Convert to trillions
      color: "#00d395",
    }));

    barSeries.setData(chartData);
    chart.timeScale().fitContent();
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
        <p class="canister-id-display">{canisterId}</p>
      </div>

      <div class="chart-container" bind:this={chartContainer}></div>

      <div class="time-range-selector">
        <button
          class="range-btn"
          class:active={timeRange === "1d"}
          on:click={() => setTimeRange("1d")}
        >
          1D
          {#if !data.is_24h_actual}<span class="no-data-indicator">*</span>{/if}
        </button>
        <button
          class="range-btn"
          class:active={timeRange === "7d"}
          on:click={() => setTimeRange("7d")}
        >
          7D
          {#if !data.is_7d_actual}<span class="no-data-indicator">*</span>{/if}
        </button>
        <button
          class="range-btn"
          class:active={timeRange === "30d"}
          on:click={() => setTimeRange("30d")}
        >
          30D
          {#if !data.is_30d_actual}<span class="no-data-indicator">*</span>{/if}
        </button>
        <span class="data-note">* extrapolated data</span>
      </div>

      <div class="stats-panel">
        <div class="stat-row">
          <span class="stat-label">Current Balance</span>
          <span class="stat-value">{formatCycles(data.current_balance)}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">1h Burn</span>
          <span class="stat-value">
            {formatCycles(data.burn_1h?.[0])}
            <span class="extrapolated-badge">~</span>
          </span>
        </div>
        <div class="stat-row">
          <span class="stat-label">24h Burn</span>
          <span class="stat-value">
            {formatCycles(data.burn_24h?.[0])}
            {#if !data.is_24h_actual}<span class="extrapolated-badge">~</span>{/if}
          </span>
        </div>
        <div class="stat-row">
          <span class="stat-label">7d Burn</span>
          <span class="stat-value">
            {formatCycles(data.burn_7d?.[0])}
            {#if !data.is_7d_actual}<span class="extrapolated-badge">~</span>{/if}
          </span>
        </div>
        <div class="stat-row">
          <span class="stat-label">30d Burn</span>
          <span class="stat-value">
            {formatCycles(data.burn_30d?.[0])}
            {#if !data.is_30d_actual}<span class="extrapolated-badge">~</span>{/if}
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

  .canister-id-display {
    color: #9ca3af;
    font-family: monospace;
    font-size: 14px;
    margin: 0;
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

  .chart-container {
    background: #1a1a2e;
    border-radius: 8px;
    margin-bottom: 16px;
    min-height: 300px;
  }

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

  .range-btn.active .no-data-indicator {
    color: #000;
  }

  .data-note {
    color: #6b7280;
    font-size: 12px;
    margin-left: auto;
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
