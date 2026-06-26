<script lang="ts">
  import type { ServerStatus } from '$lib/types/server';

  let { status }: { status: ServerStatus } = $props();

  const labels: Record<ServerStatus, string> = {
    offline: 'Offline',
    starting: 'Starting',
    online: 'Online',
    stopping: 'Stopping',
    crashed: 'Crashed',
  };
</script>

<span class="badge {status}">
  <span class="dot"></span>
  {labels[status]}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 4px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    border: 1px solid var(--border);
    background: var(--surface-2);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-faint);
  }
  .online .dot {
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }
  .online {
    color: var(--accent);
  }
  .starting .dot,
  .stopping .dot {
    background: var(--warning);
    animation: pulse 1s infinite;
  }
  .starting,
  .stopping {
    color: var(--warning);
  }
  .crashed .dot {
    background: var(--danger);
  }
  .crashed {
    color: var(--danger);
  }
  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.35;
    }
  }
</style>
