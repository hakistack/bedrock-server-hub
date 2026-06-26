<script lang="ts">
  import { api, openInFolder } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
  import StatusBadge from '$lib/components/shared/StatusBadge.svelte';
  import type { BedrockServer } from '$lib/types/server';

  const servers = $derived(serverStore.servers);
  let busy = $state(false);
  let removeTarget = $state<BedrockServer | null>(null);
  let removing = $state(false);

  async function start(s: BedrockServer) {
    if (busy) return;
    busy = true;
    try {
      await api.startServer(s.id);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }
  async function stop(s: BedrockServer) {
    if (busy) return;
    busy = true;
    try {
      await api.stopServer(s.id);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }

  async function confirmRemove() {
    if (!removeTarget) return;
    removing = true;
    try {
      await api.removeServer(removeTarget.id);
      serverStore.remove(removeTarget.id);
      toasts.success(`"${removeTarget.name}" quitado del manager.`);
      removeTarget = null;
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      removing = false;
    }
  }

  function statusOf(id: string) {
    return serverStore.statusOf(id);
  }
</script>

<PageHeader title="Servers" subtitle="Todos tus servidores Bedrock administrados.">
  {#snippet actions()}
    <Button variant="primary" href="/new">+ Nuevo servidor</Button>
  {/snippet}
</PageHeader>

{#if servers.length === 0}
  <div class="card">
    <EmptyState
      icon="🗄️"
      title="Sin servidores todavía"
      description="Crea uno nuevo o importa una carpeta existente para empezar."
    >
      {#snippet actions()}
        <Button variant="primary" href="/new">Crear o importar</Button>
      {/snippet}
    </EmptyState>
  </div>
{:else}
  <div class="list">
    {#each servers as s (s.id)}
      {@const st = statusOf(s.id)}
      <div class="server" class:selected={serverStore.selectedId === s.id}>
        <button class="main" onclick={() => serverStore.select(s.id)}>
          <div class="info">
            <div class="name-row">
              <strong>{s.name}</strong>
              {#if serverStore.selectedId === s.id}<Badge tone="success">activo</Badge>{/if}
            </div>
            <div class="meta">
              {#if s.serverVersion}<span class="mono faint">v{s.serverVersion}</span>{/if}
              {#if s.platform}<Badge>{s.platform}</Badge>{/if}
              {#if s.channel && s.channel !== 'Stable'}<Badge tone="gold">{s.channel}</Badge>{/if}
            </div>
            <div class="faint mono path">{s.path}</div>
          </div>
          <StatusBadge status={st} />
        </button>
        <div class="actions">
          {#if st === 'online' || st === 'starting'}
            <Button size="sm" onclick={() => stop(s)} disabled={busy}>■ Stop</Button>
          {:else}
            <Button size="sm" variant="primary" onclick={() => start(s)} disabled={busy}>▶ Start</Button>
          {/if}
          <Button size="sm" onclick={() => openInFolder(s.path)}>📂 Carpeta</Button>
          <Button size="sm" variant="danger" onclick={() => (removeTarget = s)}>Quitar</Button>
        </div>
      </div>
    {/each}
  </div>
{/if}

<ConfirmDialog
  open={removeTarget !== null}
  title="Quitar servidor"
  message={`"${removeTarget?.name ?? ''}" se quitará del manager.\nNo se borra ningún archivo del disco — solo se deja de administrar aquí.`}
  confirmLabel="Quitar"
  danger
  busy={removing}
  onconfirm={confirmRemove}
/>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .server {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    transition: border-color 0.12s;
  }
  .server:hover {
    border-color: var(--border-strong);
  }
  .server.selected {
    border-color: var(--accent);
  }
  .main {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    text-align: left;
    background: none;
    border: none;
    color: var(--text);
    padding: 16px 18px;
  }
  .name-row {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 7px 0;
  }
  .path {
    font-size: 12px;
    word-break: break-all;
  }
  .actions {
    display: flex;
    gap: 8px;
    padding: 0 18px 14px;
    flex-wrap: wrap;
  }
</style>
