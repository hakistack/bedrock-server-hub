<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { logsStore } from '$lib/stores/logs.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import StatusBadge from '$lib/components/shared/StatusBadge.svelte';
  import LogView from '$lib/components/shared/LogView.svelte';

  let command = $state('');

  const server = $derived(serverStore.selected);
  const lines = $derived(logsStore.get(serverStore.selectedId));
  const status = $derived(serverStore.selectedStatus);
  const canSend = $derived(status === 'online');

  async function send(e: Event) {
    e.preventDefault();
    const id = serverStore.selectedId;
    const cmd = command.trim();
    if (!id || !cmd) return;
    try {
      await api.sendServerCommand(id, cmd);
      command = '';
    } catch (err) {
      toasts.error(errorMessage(err));
    }
  }

  function clear() {
    if (serverStore.selectedId) logsStore.clear(serverStore.selectedId);
  }
</script>

<header class="page-head row spread">
  <div>
    <h1>Console</h1>
    <p class="muted">Logs en vivo del proceso del servidor.</p>
  </div>
  {#if server}
    <div class="row">
      <StatusBadge {status} />
      <button class="btn btn-sm" onclick={clear} disabled={lines.length === 0}>Limpiar vista</button>
    </div>
  {/if}
</header>

{#if !server}
  <div class="card empty-state">Selecciona o importa un servidor para ver su consola.</div>
{:else}
  <div class="console">
    <LogView {lines} height="calc(100vh - 250px)" />
    <form class="cmd" onsubmit={send}>
      <span class="prompt mono">›</span>
      <input
        class="input mono"
        placeholder={canSend ? 'Escribe un comando (ej. list, say hola)…' : 'El servidor debe estar online'}
        bind:value={command}
        disabled={!canSend}
      />
      <button class="btn btn-primary" type="submit" disabled={!canSend || !command.trim()}>
        Enviar
      </button>
    </form>
  </div>
{/if}

<style>
  .page-head {
    margin-bottom: 18px;
    align-items: flex-start;
  }
  .console {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .cmd {
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 7px 10px;
  }
  .cmd .input {
    border: none;
    background: transparent;
    flex: 1;
  }
  .cmd .input:focus {
    border: none;
  }
  .prompt {
    color: var(--accent);
    font-weight: 700;
  }
</style>
