<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { logsStore } from '$lib/stores/logs.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import StatusBadge from '$lib/components/shared/StatusBadge.svelte';
  import LogView from '$lib/components/shared/LogView.svelte';
  import type { LogLine } from '$lib/types/server';

  type Level = 'all' | 'info' | 'warning' | 'error';
  let filter = $state<Level>('all');
  let command = $state('');

  const server = $derived(serverStore.selected);
  const status = $derived(serverStore.selectedStatus);
  const canSend = $derived(status === 'online');
  const allLines = $derived(logsStore.get(serverStore.selectedId));

  function levelOf(l: LogLine): Exclude<Level, 'all'> {
    if (l.stream === 'stderr' || /\berror\b/i.test(l.line)) return 'error';
    if (/\bwarn(ing)?\b/i.test(l.line)) return 'warning';
    return 'info';
  }

  const lines = $derived(filter === 'all' ? allLines : allLines.filter((l) => levelOf(l) === filter));

  const counts = $derived({
    all: allLines.length,
    error: allLines.filter((l) => levelOf(l) === 'error').length,
    warning: allLines.filter((l) => levelOf(l) === 'warning').length,
  });

  const filters: { key: Level; label: string }[] = [
    { key: 'all', label: 'Todos' },
    { key: 'info', label: 'Info' },
    { key: 'warning', label: 'Avisos' },
    { key: 'error', label: 'Errores' },
  ];

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

<PageHeader title="Console" subtitle="Logs en vivo del proceso del servidor.">
  {#snippet actions()}
    {#if server}<StatusBadge {status} />{/if}
  {/snippet}
</PageHeader>

{#if !server}
  <div class="card"><EmptyState icon="🖥️" title="Sin servidor" description="Selecciona un servidor para ver su consola." /></div>
{:else}
  <div class="toolbar">
    <div class="seg">
      {#each filters as f (f.key)}
        <button class="seg-btn" class:active={filter === f.key} onclick={() => (filter = f.key)}>
          {f.label}
          {#if f.key === 'error' && counts.error}<span class="badge err">{counts.error}</span>
          {:else if f.key === 'warning' && counts.warning}<span class="badge warn">{counts.warning}</span>
          {:else if f.key === 'all'}<span class="badge">{counts.all}</span>{/if}
        </button>
      {/each}
    </div>
    <Button size="sm" onclick={clear} disabled={allLines.length === 0}>Limpiar vista</Button>
  </div>

  <div class="console">
    <LogView {lines} height="calc(100vh - 320px)" />
    <form class="cmd" onsubmit={send}>
      <span class="prompt mono">›</span>
      <input
        class="input mono"
        placeholder={canSend ? 'Escribe un comando (ej. list, say hola)…' : 'El servidor debe estar online'}
        bind:value={command}
        disabled={!canSend}
      />
      <Button variant="primary" type="submit" disabled={!canSend || !command.trim()}>Enviar</Button>
    </form>
  </div>
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }
  .seg {
    display: inline-flex;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 3px;
    gap: 2px;
  }
  .seg-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 5px 12px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 550;
  }
  .seg-btn.active {
    background: var(--surface-3);
    color: var(--text);
  }
  .badge {
    font-size: 10px;
    background: var(--surface-2);
    border-radius: 999px;
    padding: 1px 6px;
    color: var(--text-faint);
  }
  .badge.err {
    color: #ff8a8a;
  }
  .badge.warn {
    color: var(--warning);
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
    box-shadow: none;
  }
  .prompt {
    color: var(--accent);
    font-weight: 700;
  }
</style>
