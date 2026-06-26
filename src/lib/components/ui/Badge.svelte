<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    tone = 'default',
    children,
  }: {
    tone?: 'default' | 'success' | 'warning' | 'danger' | 'info' | 'gold';
    children?: Snippet;
  } = $props();
</script>

<span class="ui-badge t-{tone}">{@render children?.()}</span>

<style>
  .ui-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-weight: 600;
    padding: 2px 9px;
    border-radius: 999px;
    border: 1px solid var(--border);
    color: var(--text-muted);
    background: var(--surface-2);
    white-space: nowrap;
  }
  .t-success {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .t-warning {
    color: var(--warning);
    border-color: var(--warning);
    background: var(--warning-soft);
  }
  .t-danger {
    color: #ff8a8a;
    border-color: var(--danger);
    background: var(--danger-soft);
  }
  .t-gold {
    color: var(--gold);
    border-color: var(--gold);
    background: var(--gold-soft);
  }
  .t-info {
    color: var(--info);
    border-color: var(--info);
  }
</style>
