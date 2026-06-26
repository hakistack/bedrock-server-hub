<script lang="ts">
  import { api, pickFile } from '$lib/api/commands';
  import { fileDrop } from '$lib/actions/fileDrop.svelte';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { humanSize } from '$lib/util/format';
  import type { World } from '$lib/types/world';

  let worlds = $state<World[]>([]);
  let loading = $state(false);
  let busy = $state(false);
  let loadedFor = $state<string | null>(null);
  let importActive = $state(false);
  let dragHover = $state(false);

  const server = $derived(serverStore.selected);

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) load(id);
  });

  async function load(id: string) {
    loading = true;
    try {
      worlds = await api.listWorlds(id);
      loadedFor = id;
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      loading = false;
    }
  }

  async function reload() {
    if (serverStore.selectedId) await load(serverStore.selectedId);
  }

  async function doImport() {
    const path = await pickFile(['mcworld', 'zip'], 'Mundo Bedrock', 'Selecciona un .mcworld');
    if (path) await importFromPath(path);
  }

  async function importFromPath(path: string) {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      const world = await api.importWorld(id, path, importActive);
      await reload();
      toasts.success(`Mundo "${world.name}" importado${importActive ? ' y activado' : ''}.`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }

  async function activate(world: World) {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.activateWorld(id, world.name);
      await reload();
      toasts.success(`"${world.name}" es ahora el mundo activo.`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }

  async function backup(world: World) {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.createBackup(id, world.name);
      toasts.success(`Backup de "${world.name}" creado.`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }
</script>

<header class="page-head row spread">
  <div>
    <h1>Worlds</h1>
    <p class="muted">Mundos detectados en <span class="mono">worlds/</span>.</p>
  </div>
  {#if server}
    <div class="import">
      <label class="chk">
        <input type="checkbox" bind:checked={importActive} />
        Activar al importar
      </label>
      <button class="btn btn-primary" onclick={doImport} disabled={busy}>
        {busy ? 'Trabajando…' : '+ Importar .mcworld'}
      </button>
    </div>
  {/if}
</header>

{#if !server}
  <div class="card empty-state">Selecciona o importa un servidor para gestionar sus mundos.</div>
{:else}
  <div
    class="drop-wrap"
    class:drag-hover={dragHover}
    use:fileDrop={{
      extensions: ['mcworld', 'zip'],
      onDrop: importFromPath,
      onHover: (h) => (dragHover = h),
    }}
  >
    {#if dragHover}
      <div class="drop-banner">Suelta el <span class="mono">.mcworld</span> para importarlo</div>
    {/if}
    {#if loading}
      <div class="card muted">Cargando mundos…</div>
    {:else if worlds.length === 0}
      <div class="card empty-state">
        <h2>🌍 Sin mundos todavía</h2>
        <p class="muted">
          Importa un <span class="mono">.mcworld</span> (botón o arrastrándolo aquí) para empezar.
        </p>
      </div>
    {:else}
      <div class="world-list">
    {#each worlds as w (w.name)}
      <div class="card world" class:active={w.isActive}>
        <div class="world-main">
          <div class="row" style="gap:10px;">
            <h3>{w.displayName ?? w.name}</h3>
            {#if w.isActive}<span class="active-tag">Activo</span>{/if}
          </div>
          <p class="faint mono small">{w.name} · {humanSize(w.sizeBytes)}</p>
          <div class="tags">
            {#if w.hasBehaviorPacks}<span class="tag">behavior packs</span>{/if}
            {#if w.hasResourcePacks}<span class="tag">resource packs</span>{/if}
          </div>
        </div>
        <div class="world-actions">
          <button class="btn btn-sm" onclick={() => backup(w)} disabled={busy}>💾 Backup</button>
          <button
            class="btn btn-sm btn-primary"
            onclick={() => activate(w)}
            disabled={busy || w.isActive}
          >
            {w.isActive ? 'Activo' : 'Activar'}
          </button>
        </div>
      </div>
    {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .page-head {
    margin-bottom: 22px;
    align-items: flex-start;
  }
  .import {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    color: var(--text-muted);
  }
  .chk input {
    accent-color: var(--accent);
  }
  .drop-wrap {
    border-radius: var(--radius);
    transition: outline-color 0.15s;
    outline: 2px dashed transparent;
    outline-offset: 6px;
  }
  .drop-wrap.drag-hover {
    outline-color: var(--accent);
  }
  .drop-banner {
    background: rgba(59, 165, 93, 0.12);
    border: 1px solid var(--accent);
    color: var(--accent);
    border-radius: var(--radius-sm);
    padding: 12px;
    text-align: center;
    margin-bottom: 12px;
    font-weight: 500;
  }
  .world-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .world {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .world.active {
    border-color: var(--accent);
  }
  .active-tag {
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: 999px;
    padding: 1px 9px;
  }
  .small {
    font-size: 12px;
    margin: 6px 0 0;
  }
  .tags {
    display: flex;
    gap: 6px;
    margin-top: 9px;
  }
  .tag {
    font-size: 11px;
    color: var(--text-muted);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px 8px;
  }
  .world-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }
</style>
