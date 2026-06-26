<script lang="ts">
  import { api, pickFile, openInFolder } from '$lib/api/commands';
  import { fileDrop } from '$lib/actions/fileDrop.svelte';
  import { goto } from '$app/navigation';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { humanSize, formatDate } from '$lib/util/format';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
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
  async function activate(w: World) {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.activateWorld(id, w.name);
      await reload();
      toasts.success(`"${w.name}" es ahora el mundo activo.`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }
  async function backup(w: World) {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.createBackup(id, w.name);
      toasts.success(`Backup de "${w.name}" creado.`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }
  function packCount(w: World): number {
    return (w.hasBehaviorPacks ? 1 : 0) + (w.hasResourcePacks ? 1 : 0);
  }
</script>

<PageHeader title="Worlds" subtitle="Mundos detectados en la carpeta worlds/.">
  {#snippet actions()}
    {#if server}
      <label class="chk">
        <input type="checkbox" bind:checked={importActive} />
        Activar al importar
      </label>
      <Button variant="primary" onclick={doImport} loading={busy}>+ Importar .mcworld</Button>
    {/if}
  {/snippet}
</PageHeader>

{#if !server}
  <div class="card"><EmptyState icon="🌍" title="Sin servidor" description="Selecciona un servidor para gestionar sus mundos." /></div>
{:else}
  <div
    class="drop-wrap"
    class:drag-hover={dragHover}
    use:fileDrop={{ extensions: ['mcworld', 'zip'], onDrop: importFromPath, onHover: (h) => (dragHover = h) }}
  >
    {#if dragHover}
      <div class="drop-banner">Suelta el <span class="mono">.mcworld</span> para importarlo</div>
    {/if}
    {#if loading}
      <div class="card"><Spinner text="Cargando mundos…" /></div>
    {:else if worlds.length === 0}
      <div class="card">
        <EmptyState icon="🌍" title="Sin mundos todavía" description="Importa un .mcworld (botón o arrastrándolo aquí) para empezar." />
      </div>
    {:else}
      <div class="grid">
        {#each worlds as w (w.name)}
          <div class="world" class:active={w.isActive}>
            <div class="w-head">
              <h3>{w.displayName ?? w.name}</h3>
              {#if w.isActive}<Badge tone="success">Activo</Badge>{/if}
            </div>
            <div class="w-meta">
              <span class="mono faint">{w.name}</span>
              <span class="dot">·</span>
              <span>{humanSize(w.sizeBytes)}</span>
              {#if w.modifiedAt}<span class="dot">·</span><span class="faint">{formatDate(w.modifiedAt)}</span>{/if}
            </div>
            <div class="tags">
              {#if w.hasBehaviorPacks}<Badge tone="info">behavior</Badge>{/if}
              {#if w.hasResourcePacks}<Badge tone="success">resource</Badge>{/if}
              {#if packCount(w) === 0}<span class="faint small">sin packs</span>{/if}
            </div>
            <div class="w-actions">
              <Button size="sm" variant="primary" onclick={() => activate(w)} disabled={busy || w.isActive}>
                {w.isActive ? 'Activo' : 'Activar'}
              </Button>
              <Button size="sm" onclick={() => backup(w)} disabled={busy}>💾 Backup</Button>
              <Button size="sm" onclick={() => openInFolder(w.path)}>📂 Carpeta</Button>
              <Button size="sm" onclick={() => goto('/addons')}>🧩 Packs</Button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
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
    outline: 2px dashed transparent;
    outline-offset: 6px;
    transition: outline-color 0.15s;
  }
  .drop-wrap.drag-hover {
    outline-color: var(--accent);
  }
  .drop-banner {
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    color: var(--accent);
    border-radius: var(--radius-sm);
    padding: 12px;
    text-align: center;
    margin-bottom: 12px;
    font-weight: 550;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 14px;
  }
  .world {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px 18px;
  }
  .world.active {
    border-color: var(--accent);
  }
  .w-head {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .w-head h3 {
    font-size: 16px;
  }
  .w-meta {
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 8px 0;
    font-size: 12px;
    color: var(--text-muted);
    flex-wrap: wrap;
  }
  .dot {
    color: var(--text-faint);
  }
  .tags {
    display: flex;
    gap: 6px;
    margin-bottom: 14px;
    min-height: 20px;
  }
  .small {
    font-size: 12px;
  }
  .w-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
