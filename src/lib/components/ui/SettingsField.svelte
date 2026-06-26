<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    description,
    hint,
    children,
  }: {
    label: string;
    description?: string;
    hint?: string;
    children?: Snippet;
  } = $props();
</script>

<div class="ui-field">
  <div class="head">
    <span class="label">{label}</span>
    {#if hint}<span class="hint mono">{hint}</span>{/if}
  </div>
  <div class="control">{@render children?.()}</div>
  {#if description}<p class="desc">{description}</p>{/if}
</div>

<style>
  .ui-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }
  .label {
    font-size: 13px;
    font-weight: 550;
  }
  .hint {
    font-size: 11px;
    color: var(--text-faint);
  }
  .desc {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.4;
  }
</style>
