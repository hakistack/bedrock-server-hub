<script lang="ts">
  import { fileDrop } from '$lib/actions/fileDrop.svelte';
  import { pickFiles } from '$lib/api/commands';

  let {
    extensions,
    name,
    title = 'Selecciona archivos',
    icon = '📁',
    label = 'Arrastra o selecciona archivos',
    hint,
    busy = false,
    onFile,
  }: {
    extensions: string[];
    name: string;
    title?: string;
    icon?: string;
    label?: string;
    hint?: string;
    busy?: boolean;
    onFile: (path: string) => void;
  } = $props();

  let hover = $state(false);

  async function pick() {
    if (busy) return;
    const paths = await pickFiles(extensions, name, title);
    for (const p of paths) onFile(p);
  }
</script>

<button
  class="dz"
  class:hover
  class:busy
  onclick={pick}
  use:fileDrop={{ extensions, onDrop: onFile, onHover: (h) => (hover = h) }}
>
  <div class="icon">{icon}</div>
  {#if busy}
    <p class="title">Analizando…</p>
  {:else if hover}
    <p class="title">Suelta aquí</p>
  {:else}
    <p class="title">{label}</p>
    {#if hint}<p class="hint faint">{hint}</p>{/if}
  {/if}
</button>

<style>
  .dz {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 30px 24px;
    border: 1.5px dashed var(--border-strong);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    transition: border-color 0.15s, background 0.15s;
  }
  .dz:hover,
  .dz.hover {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .icon {
    font-size: 30px;
  }
  .title {
    margin: 2px 0 0;
    font-weight: 550;
  }
  .hint {
    margin: 0;
    font-size: 12px;
  }
</style>
