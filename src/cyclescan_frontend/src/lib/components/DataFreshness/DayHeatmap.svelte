<script>
  export let days = [];
  export let snapshotsByDay = new Map();
  export let selectedDayIndex = 6;
  export let onSelect = () => {};

  function getCompleteness(dateKey, index) {
    const snaps = snapshotsByDay.get(dateKey) || [];
    const count = snaps.length;

    // For today, calculate expected snapshots
    const isToday = index === days.length - 1;
    const expected = isToday ? new Date().getHours() + 1 : 24;

    const ratio = count / expected;

    if (count === 0) return 'empty';
    if (ratio >= 1) return 'full';
    if (ratio >= 0.75) return 'mostly';
    if (ratio >= 0.5) return 'partial';
    return 'sparse';
  }

  function getTooltip(day, index) {
    const snaps = snapshotsByDay.get(day.dateKey) || [];
    const isToday = index === days.length - 1;
    const expected = isToday ? new Date().getHours() + 1 : 24;

    const dateStr = day.date.toLocaleDateString('en-US', {
      weekday: 'long',
      month: 'short',
      day: 'numeric'
    });

    return `${dateStr}\n${snaps.length}/${expected} snapshots`;
  }
</script>

<div class="day-heatmap">
  {#each days as day, i}
    {@const completeness = getCompleteness(day.dateKey, i)}
    <button
      class="day-cell {completeness}"
      class:selected={i === selectedDayIndex}
      class:today={i === days.length - 1}
      on:click={() => onSelect(i)}
      title={getTooltip(day, i)}
    >
      <span class="day-name">{day.dayName}</span>
      <span class="day-bar"></span>
    </button>
  {/each}
</div>

<style>
  .day-heatmap {
    display: flex;
    gap: 4px;
  }

  .day-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .day-cell:hover {
    background: var(--bg-hover, rgba(255,255,255,0.05));
  }

  .day-cell.selected {
    border-color: var(--color-primary, #4ade80);
    background: var(--bg-selected, rgba(74, 222, 128, 0.1));
  }

  .day-name {
    font-size: 11px;
    color: var(--text-muted, #888);
  }

  .day-bar {
    width: 24px;
    height: 6px;
    border-radius: 2px;
    background: var(--color-empty, #333);
  }

  /* Completeness colors */
  .day-cell.full .day-bar {
    background: var(--color-full, #4ade80);
  }

  .day-cell.mostly .day-bar {
    background: var(--color-mostly, #a3e635);
  }

  .day-cell.partial .day-bar {
    background: var(--color-partial, #fbbf24);
  }

  .day-cell.sparse .day-bar {
    background: var(--color-sparse, #f87171);
  }

  .day-cell.empty .day-bar {
    background: var(--color-empty, #333);
  }

  /* Today marker */
  .day-cell.today .day-name {
    font-weight: 600;
    color: var(--text-primary, #fff);
  }
</style>
