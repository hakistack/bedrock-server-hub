<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    subtitle,
    actions,
  }: {
    title: string;
    subtitle?: string;
    actions?: Snippet;
  } = $props();
</script>

<header class="ui-page-header">
  <div class="titles">
    <h1>{title}</h1>
    {#if subtitle}<p class="sub">{subtitle}</p>{/if}
  </div>
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
  .ui-page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 24px;
  }
  h1 {
    font-size: 26px;
  }
  .sub {
    margin: 5px 0 0;
    color: var(--text-muted);
    font-size: 14px;
  }
  .actions {
    display: flex;
    gap: 10px;
    flex-shrink: 0;
    flex-wrap: wrap;
    justify-content: flex-end;
  }
</style>
