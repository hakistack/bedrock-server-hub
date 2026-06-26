<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    elevated = false,
    padding = 'md',
    children,
    actions,
  }: {
    title?: string;
    elevated?: boolean;
    padding?: 'sm' | 'md' | 'lg' | 'none';
    children?: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<section class="ui-card p-{padding}" class:elevated>
  {#if title || actions}
    <header class="ui-card-head">
      {#if title}<span class="ui-card-title">{title}</span>{/if}
      {#if actions}<div class="ui-card-actions">{@render actions()}</div>{/if}
    </header>
  {/if}
  {@render children?.()}
</section>

<style>
  .ui-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }
  .ui-card.elevated {
    box-shadow: var(--shadow);
  }
  .p-none {
    padding: 0;
  }
  .p-sm {
    padding: 14px;
  }
  .p-md {
    padding: 20px;
  }
  .p-lg {
    padding: 28px;
  }
  .ui-card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 14px;
  }
  .ui-card-title {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    font-weight: 600;
  }
  .ui-card-actions {
    display: flex;
    gap: 8px;
  }
</style>
