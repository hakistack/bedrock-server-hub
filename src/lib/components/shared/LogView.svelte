<script lang="ts">
  import { tick } from 'svelte';
  import type { LogLine } from '$lib/types/server';

  let { lines, height = '100%' }: { lines: LogLine[]; height?: string } = $props();

  let container = $state<HTMLDivElement | null>(null);
  let pinned = $state(true);

  // Auto-scroll to the bottom when new lines arrive, unless the user scrolled up.
  $effect(() => {
    void lines.length;
    if (pinned && container) {
      tick().then(() => {
        if (container) container.scrollTop = container.scrollHeight;
      });
    }
  });

  function onScroll() {
    if (!container) return;
    const nearBottom =
      container.scrollHeight - container.scrollTop - container.clientHeight < 40;
    pinned = nearBottom;
  }
</script>

<div class="log" bind:this={container} onscroll={onScroll} style:height>
  {#if lines.length === 0}
    <div class="log-empty faint">Sin logs todavía. Inicia el servidor para ver la salida.</div>
  {:else}
    {#each lines as l, i (i)}
      <div class="line" class:err={l.stream === 'stderr'}>{l.line}</div>
    {/each}
  {/if}
</div>

<style>
  .log {
    background: #0a0c10;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px 14px;
    overflow-y: auto;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12.5px;
    line-height: 1.55;
  }
  .line {
    white-space: pre-wrap;
    word-break: break-word;
    color: #c9d1d9;
  }
  .line.err {
    color: #ff9a9a;
  }
  .log-empty {
    padding: 18px 4px;
  }
</style>
