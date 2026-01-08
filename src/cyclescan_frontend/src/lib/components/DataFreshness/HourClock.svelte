<script>
  export let snapshots = [];
  export let day = { dateKey: '', date: new Date() };
  export let isToday = false;

  const CLOCK_SIZE = 120;
  const CENTER = CLOCK_SIZE / 2;
  const RADIUS = 45;
  const DOT_RADIUS = 4;

  // Current hour (for today's view)
  $: currentHour = new Date().getHours();

  // Build set of hours that have snapshots
  $: hoursWithData = new Set(
    (snapshots || []).map(s => new Date(s.timestamp).getHours())
  );

  // Generate clock positions for 24 hours
  // 0 (midnight) at top, going clockwise
  function getHourPosition(hour) {
    // Convert hour to angle (0 = top, clockwise)
    // 0h = -90deg, 6h = 0deg, 12h = 90deg, 18h = 180deg
    const angle = ((hour / 24) * 360 - 90) * (Math.PI / 180);
    return {
      x: CENTER + RADIUS * Math.cos(angle),
      y: CENTER + RADIUS * Math.sin(angle),
    };
  }

  // Pre-compute hour statuses for reactivity (forces SVG re-render)
  // Must come after hoursWithData and use it directly to establish dependency
  $: hourStatuses = Array.from({ length: 24 }, (_, hour) => {
    const pos = getHourPosition(hour);
    let status;
    if (isToday && hour > currentHour) {
      status = 'future';
    } else if (hoursWithData && hoursWithData.has(hour)) {
      status = 'has-data';
    } else {
      status = 'missing';
    }

    let tooltip;
    const timeStr = `${hour.toString().padStart(2, '0')}:00`;
    if (status === 'future') {
      tooltip = `${timeStr} - upcoming`;
    } else if (status === 'has-data') {
      const snap = (snapshots || []).find(s => new Date(s.timestamp).getHours() === hour);
      if (snap) {
        const exactTime = new Date(snap.timestamp).toLocaleTimeString('en-US', {
          hour: '2-digit',
          minute: '2-digit',
        });
        tooltip = `${exactTime} - collected`;
      } else {
        tooltip = `${timeStr} - collected`;
      }
    } else {
      tooltip = `${timeStr} - missing`;
    }

    return { hour, pos, status, tooltip };
  });

  // Hour labels (only show 0, 6, 12, 18)
  const hourLabels = [
    { hour: 0, label: '0', pos: getHourPosition(0), offset: { x: 0, y: -10 } },
    { hour: 6, label: '6', pos: getHourPosition(6), offset: { x: 10, y: 0 } },
    { hour: 12, label: '12', pos: getHourPosition(12), offset: { x: 0, y: 12 } },
    { hour: 18, label: '18', pos: getHourPosition(18), offset: { x: -14, y: 0 } },
  ];
</script>

<div class="hour-clock">
  <svg width={CLOCK_SIZE} height={CLOCK_SIZE} viewBox="0 0 {CLOCK_SIZE} {CLOCK_SIZE}">
    <!-- Clock face circle -->
    <circle
      cx={CENTER}
      cy={CENTER}
      r={RADIUS + 8}
      fill="none"
      stroke="var(--border-subtle, #333)"
      stroke-width="1"
    />

    <!-- Center dot -->
    <circle
      cx={CENTER}
      cy={CENTER}
      r="3"
      fill="var(--text-muted, #666)"
    />

    <!-- Hour dots -->
    {#each hourStatuses as { hour, pos, status, tooltip } (hour)}
      <circle
        cx={pos.x}
        cy={pos.y}
        r={status === 'future' ? DOT_RADIUS - 1.5 : DOT_RADIUS}
        class="hour-dot {status}"
        data-hour={hour}
      >
        <title>{tooltip}</title>
      </circle>
    {/each}

    <!-- Hour labels -->
    {#each hourLabels as { label, pos, offset }}
      <text
        x={pos.x + offset.x}
        y={pos.y + offset.y}
        class="hour-label"
        text-anchor="middle"
        dominant-baseline="middle"
      >
        {label}
      </text>
    {/each}

    <!-- Current hour hand (today only) -->
    {#if isToday}
      {@const handPos = getHourPosition(currentHour)}
      <line
        x1={CENTER}
        y1={CENTER}
        x2={CENTER + (handPos.x - CENTER) * 0.6}
        y2={CENTER + (handPos.y - CENTER) * 0.6}
        class="hour-hand"
        stroke-width="2"
        stroke-linecap="round"
      />
    {/if}
  </svg>
</div>

<style>
  .hour-clock {
    display: flex;
    justify-content: center;
    padding: 8px;
  }

  .hour-dot {
    transition: all 0.15s ease;
    cursor: pointer;
  }

  .hour-dot.has-data {
    fill: var(--color-full, #4ade80);
  }

  .hour-dot.missing {
    fill: transparent;
    stroke: var(--color-sparse, #f87171);
    stroke-width: 1.5;
  }

  .hour-dot.future {
    fill: var(--text-muted, #444);
    opacity: 0.4;
  }

  .hour-label {
    font-size: 10px;
    fill: var(--text-muted, #888);
    font-family: inherit;
  }

  .hour-hand {
    stroke: var(--color-primary, #4ade80);
    opacity: 0.7;
  }
</style>
