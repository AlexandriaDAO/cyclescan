<script>
  export let intervals = [];
  export let width = 80;
  export let height = 20;
  export let barGap = 1;
  export let showInferred = true;  // Whether to color inferred bars differently

  // Show last 24 hours (one bar per hour)
  const MAX_BARS = 24;

  $: displayIntervals = intervals.slice(-MAX_BARS);

  // Get the burn value for each interval (actual or inferred)
  $: burns = displayIntervals.map(i => i.actualBurn + i.inferredBurn);

  // Find max for scaling (avoid division by zero)
  $: maxBurn = Math.max(...burns, 1);

  // Calculate median for spike detection
  $: medianBurn = (() => {
    if (burns.length === 0) return 0;
    const sorted = [...burns].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    return sorted.length % 2 !== 0 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
  })();

  // Use logarithmic scale if data is very spiky
  $: useLogScale = maxBurn > medianBurn * 10 && medianBurn > 0;

  // Calculate bar dimensions
  $: barWidth = Math.max(1, (width - (displayIntervals.length - 1) * barGap) / Math.max(displayIntervals.length, 1));

  // Build bar data
  $: bars = displayIntervals.map((interval, i) => {
    const burn = interval.actualBurn + interval.inferredBurn;
    let barHeight;

    if (useLogScale) {
      barHeight = (Math.log10(burn + 1) / Math.log10(maxBurn + 1)) * height;
    } else {
      barHeight = (burn / maxBurn) * height;
    }

    return {
      x: i * (barWidth + barGap),
      y: height - barHeight,
      width: barWidth,
      height: Math.max(1, barHeight),  // Minimum 1px height for visibility
      isInferred: interval.isTopUp,
      burn,
    };
  });

  // Check if all burns are zero
  $: allZero = burns.every(b => b === 0);

  // Check if we have insufficient data
  $: insufficientData = displayIntervals.length < 3;

  // Compute stats for tooltip
  $: avgBurn = burns.length > 0 ? burns.reduce((a, b) => a + b, 0) / burns.length : 0;
  $: hasInferredData = displayIntervals.some(i => i.isTopUp);
</script>

{#if insufficientData}
  <span class="sparkline-no-data" title="Insufficient data">—</span>
{:else if allZero}
  <span class="sparkline-zero" title="No activity">·</span>
{:else}
  <div
    class="sparkline-container"
    title={`${displayIntervals.length}h trend | Avg: ${Math.round(avgBurn / 1e9)}B/h | Peak: ${Math.round(maxBurn / 1e9)}B/h${hasInferredData ? ' | *includes inferred data' : ''}`}
  >
    <svg {width} {height} class="sparkline">
      {#each bars as bar}
        <rect
          x={bar.x}
          y={bar.y}
          width={bar.width}
          height={bar.height}
          class:inferred={showInferred && bar.isInferred}
          class:actual={!showInferred || !bar.isInferred}
        />
      {/each}
    </svg>
  </div>
{/if}

<style>
  .sparkline-container {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .sparkline {
    display: block;
  }

  .sparkline rect.actual {
    fill: var(--color-burn, #4ade80);
  }

  .sparkline rect.inferred {
    fill: var(--color-inferred, #fbbf24);
  }

  .sparkline-no-data,
  .sparkline-zero {
    color: var(--text-muted, #a19b88);
    font-size: 13px;
  }
</style>
