<script>
  import DayHeatmap from './DayHeatmap.svelte';
  import HourClock from './HourClock.svelte';
  import LiveIndicator from './LiveIndicator.svelte';

  export let snapshots = [];
  export let compact = false;

  // Expanded state for non-compact mode
  let expanded = false;

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

<div class="data-freshness" class:compact class:expanded>
  {#if compact}
    <DayHeatmap
      days={last7Days}
      {snapshotsByDay}
      {selectedDayIndex}
      onSelect={handleSelectDay}
      compact={true}
    />
    <LiveIndicator {minutesAgo} />
  {:else}
    <button class="freshness-toggle" on:click={() => expanded = !expanded}>
      <div class="toggle-header">
        <DayHeatmap
          days={last7Days}
          {snapshotsByDay}
          {selectedDayIndex}
          onSelect={handleSelectDay}
          compact={true}
        />
        <LiveIndicator {minutesAgo} />
        <span class="expand-icon" class:expanded>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </span>
      </div>
    </button>

    {#if expanded && selectedDay}
      <div class="expanded-content">
        <DayHeatmap
          days={last7Days}
          {snapshotsByDay}
          {selectedDayIndex}
          onSelect={handleSelectDay}
        />

        <HourClock
          snapshots={selectedDaySnapshots}
          day={selectedDay}
          isToday={selectedDayIndex === 6}
        />

        <div class="day-summary">
          {selectedDay.date.toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' })}
          · {selectedDaySnapshots.length} of {selectedDayIndex === 6 ? new Date().getHours() + 1 : 24} snapshots
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .data-freshness {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .data-freshness.compact {
    flex-direction: row;
    gap: 8px;
    padding: 0;
    background: transparent;
    border-radius: 0;
  }

  .freshness-toggle {
    display: flex;
    flex-direction: column;
    align-items: center;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 8px 12px;
    border-radius: 6px;
    transition: background 0.15s ease;
  }

  .freshness-toggle:hover {
    background: var(--bg-hover, rgba(255,255,255,0.05));
  }

  .toggle-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .expand-icon {
    display: flex;
    align-items: center;
    color: var(--text-muted, #888);
    transition: transform 0.2s ease;
  }

  .expand-icon.expanded {
    transform: rotate(180deg);
  }

  .expanded-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px;
    background: var(--bg-secondary, #1a1a2e);
    border-radius: 8px;
    margin-top: 4px;
  }

  .day-summary {
    font-size: 11px;
    color: var(--text-secondary, #aaa);
  }
</style>
