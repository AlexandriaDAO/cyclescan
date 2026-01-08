<script>
  export let minutesAgo = 0;

  $: status = getStatus(minutesAgo);
  $: displayText = formatMinutesAgo(minutesAgo);

  function getStatus(mins) {
    if (mins < 120) return 'live';      // < 2 hours
    if (mins < 240) return 'stale';     // 2-4 hours
    return 'error';                      // > 4 hours
  }

  function formatMinutesAgo(mins) {
    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }
</script>

<div class="live-indicator {status}">
  <span class="pulse-dot"></span>
  <span class="live-text">
    {#if status === 'live'}
      Live
    {:else if status === 'stale'}
      Stale
    {:else}
      Offline
    {/if}
    ({displayText})
  </span>
</div>

<style>
  .live-indicator {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }

  .live-indicator.live {
    background: rgba(74, 222, 128, 0.15);
    color: var(--color-full, #4ade80);
  }

  .live-indicator.stale {
    background: rgba(251, 191, 36, 0.15);
    color: var(--color-partial, #fbbf24);
  }

  .live-indicator.error {
    background: rgba(248, 113, 113, 0.15);
    color: var(--color-sparse, #f87171);
  }

  .pulse-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
  }

  .live-indicator.live .pulse-dot {
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(0.85); }
  }

  .live-text {
    white-space: nowrap;
  }
</style>
