<script>
  import DayHeatmap from './DayHeatmap.svelte';
  import HourClock from './HourClock.svelte';
  import LiveIndicator from './LiveIndicator.svelte';

  export let snapshots = [];
  export let compact = false;

  // Group snapshots by day
  $: snapshotsByDay = groupByDay(snapshots);

  // Get last 7 days (newest last)
  $: last7Days = getLast7Days();

  // Selected day (default to today)
  let selectedDayIndex = 6;  // Last day (today)

  $: selectedDay = last7Days[selectedDayIndex];
  $: selectedDaySnapshots = snapshotsByDay.get(selectedDay?.dateKey) || [];

  // Freshness calculation
  $: latestTimestamp = snapshots[0]?.timestamp || 0;
  $: minutesAgo = Math.floor((Date.now() - latestTimestamp) / 60000);

  // Get local date key in YYYY-MM-DD format (consistent for grouping)
  function getLocalDateKey(date) {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }

  function groupByDay(snaps) {
    const groups = new Map();

    for (const snap of snaps) {
      const date = new Date(snap.timestamp);
      const dateKey = getLocalDateKey(date);

      if (!groups.has(dateKey)) {
        groups.set(dateKey, []);
      }
      groups.get(dateKey).push(snap);
    }

    return groups;
  }

  function getLast7Days() {
    const days = [];
    const now = new Date();

    for (let i = 6; i >= 0; i--) {
      const date = new Date(now);
      date.setDate(date.getDate() - i);
      date.setHours(0, 0, 0, 0);

      days.push({
        dateKey: getLocalDateKey(date),
        dayName: date.toLocaleDateString('en-US', { weekday: 'short' }),
        date,
      });
    }

    return days;
  }

  function handleSelectDay(index) {
    selectedDayIndex = index;
  }
</script>

<div class="data-freshness" class:compact>
  <div class="freshness-header">
    <span class="freshness-title">Data Coverage</span>
    <LiveIndicator {minutesAgo} />
  </div>

  <DayHeatmap
    days={last7Days}
    {snapshotsByDay}
    {selectedDayIndex}
    onSelect={handleSelectDay}
  />

  {#if !compact && selectedDay}
    <HourClock
      snapshots={selectedDaySnapshots}
      day={selectedDay}
      isToday={selectedDayIndex === 6}
    />

    <div class="day-summary">
      {selectedDay.date.toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' })}
      · {selectedDaySnapshots.length} of {selectedDayIndex === 6 ? new Date().getHours() + 1 : 24} snapshots
    </div>
  {/if}
</div>

<style>
  .data-freshness {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: var(--bg-secondary, #1a1a2e);
    border-radius: 8px;
  }

  .freshness-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }

  .freshness-title {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted, #888);
  }

  .day-summary {
    font-size: 13px;
    color: var(--text-secondary, #aaa);
  }

  .compact .freshness-header {
    justify-content: center;
    gap: 12px;
  }
</style>
