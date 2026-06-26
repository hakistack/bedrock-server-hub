<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    icon = '📦',
    title,
    description,
    actions,
    compact = false,
  }: {
    icon?: string;
    title: string;
    description?: string;
    actions?: Snippet;
    compact?: boolean;
  } = $props();
</script>

<div class="ui-empty" class:compact>
  <div class="icon">{icon}</div>
  <h2>{title}</h2>
  {#if description}<p class="desc">{description}</p>{/if}
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</div>

<style>
  .ui-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 8px;
    padding: 56px 24px;
  }
  .ui-empty.compact {
    padding: 32px 20px;
  }
  .icon {
    font-size: 40px;
    margin-bottom: 4px;
    filter: saturate(0.9);
  }
  h2 {
    font-size: 18px;
  }
  .desc {
    margin: 0;
    color: var(--text-muted);
    max-width: 420px;
  }
  .actions {
    display: flex;
    gap: 10px;
    margin-top: 12px;
    flex-wrap: wrap;
    justify-content: center;
  }
</style>
