<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { logsStore } from '$lib/stores/logs.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import StatusBadge from '$lib/components/shared/StatusBadge.svelte';
  import ServerControls from '$lib/components/server/ServerControls.svelte';
  import LogView from '$lib/components/shared/LogView.svelte';
  import Sparkline from '$lib/components/shared/Sparkline.svelte';
  import { metricsStore } from '$lib/stores/metrics.store.svelte';
  import { humanSize } from '$lib/util/format';

  const server = $derived(serverStore.selected);
  // Last lines of the active server for a quick glance.
  const recentLogs = $derived(logsStore.get(serverStore.selectedId).slice(-12));

  const metrics = $derived(metricsStore.get(serverStore.selectedId));
  const cpuValues = $derived(metrics.map((m) => m.cpu));
  const memValues = $derived(metrics.map((m) => m.memoryBytes));
  const latest = $derived(metrics.at(-1));

  // Crash auto-restart preference, loaded per selected server.
  let autoRestart = $state(false);
  let autoRestartLoadedFor = $state<string | null>(null);

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== autoRestartLoadedFor) {
      autoRestartLoadedFor = id;
      api
        .getServerSettings(id)
        .then((s) => (autoRestart = s.autoRestart))
        .catch(() => (autoRestart = false));
    }
  });

  async function toggleAutoRestart(e: Event) {
    const id = serverStore.selectedId;
    if (!id) return;
    const enabled = (e.target as HTMLInputElement).checked;
    autoRestart = enabled;
    try {
      await api.setAutoRestart(id, enabled);
    } catch (err) {
      autoRestart = !enabled;
      toasts.error(errorMessage(err));
    }
  }
</script>

<header class="page-head">
  <div>
    <h1>Dashboard</h1>
    <p class="muted">Estado y control de tu servidor Bedrock.</p>
  </div>
</header>

{#if !server}
  <div class="card empty-state">
    <h2>No hay servidores todavía</h2>
    <p class="muted">Crea uno nuevo: importa una carpeta existente o descarga el oficial.</p>
    <div style="margin-top:16px; display:flex; justify-content:center;">
      <a class="btn btn-primary" href="/new">+ Nuevo servidor</a>
    </div>
  </div>
{:else}
  <div class="grid">
    <section class="card">
      <div class="row spread">
        <div>
          <div class="card-title">Servidor activo</div>
          <h2>{server.name}</h2>
          <p class="faint mono path">{server.path}</p>
        </div>
        <StatusBadge status={serverStore.selectedStatus} />
      </div>
      <div style="margin-top:18px;">
        <ServerControls />
      </div>
      <label class="auto-restart">
        <input type="checkbox" checked={autoRestart} onchange={toggleAutoRestart} />
        <span>Reiniciar automáticamente tras un crash</span>
      </label>
    </section>

    <section class="card">
      <div class="card-title">Accesos rápidos</div>
      <div class="quick">
        <button class="quick-btn" onclick={() => goto('/settings')}>
          ⚙️ Editar server.properties
        </button>
        <button class="quick-btn" onclick={() => goto('/console')}>
          🖥️ Abrir consola
        </button>
        <button class="quick-btn" onclick={() => goto('/worlds')}>
          🌍 Mundos / importar .mcworld
        </button>
        <button class="quick-btn" onclick={() => goto('/backups')}>
          💾 Backups
        </button>
        <button class="quick-btn" onclick={() => goto('/addons')}>
          🧩 Instalar addon
        </button>
      </div>
    </section>
  </div>

  <section class="card metrics-card">
    <div class="card-title">Rendimiento del proceso</div>
    {#if metrics.length === 0}
      <p class="muted small">Sin datos. Inicia el servidor para ver CPU y memoria en vivo.</p>
    {:else}
      <div class="metrics-grid">
        <div>
          <div class="metric-head">
            <span class="muted small">CPU</span>
            <span class="metric-val">{latest ? latest.cpu.toFixed(0) : 0}%</span>
          </div>
          <Sparkline values={cpuValues} color="#6fb1ff" />
        </div>
        <div>
          <div class="metric-head">
            <span class="muted small">Memoria</span>
            <span class="metric-val">{latest ? humanSize(latest.memoryBytes) : '—'}</span>
          </div>
          <Sparkline values={memValues} color="var(--accent)" />
        </div>
      </div>
    {/if}
  </section>

  <section class="card logs-card">
    <div class="row spread">
      <div class="card-title" style="margin:0;">Últimos logs</div>
      <a class="muted small" href="/console">Ver consola completa →</a>
    </div>
    <div style="margin-top:12px;">
      <LogView lines={recentLogs} height="260px" />
    </div>
  </section>
{/if}

<style>
  .page-head {
    margin-bottom: 22px;
  }
  .grid {
    display: grid;
    grid-template-columns: 1.4fr 1fr;
    gap: 18px;
  }
  .path {
    margin: 6px 0 0;
    font-size: 12px;
    word-break: break-all;
  }
  .quick {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .quick-btn {
    text-align: left;
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 11px 13px;
    border-radius: var(--radius-sm);
    transition: background 0.12s;
  }
  .quick-btn:hover {
    background: #2b313d;
  }
  .auto-restart {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: 16px;
    font-size: 13px;
    color: var(--text-muted);
  }
  .auto-restart input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .metrics-card {
    margin-top: 18px;
  }
  .metrics-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
  }
  .metric-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 7px;
  }
  .metric-val {
    font-family: ui-monospace, monospace;
    font-weight: 600;
  }
  .small {
    font-size: 12px;
  }
  .logs-card {
    margin-top: 18px;
  }
  @media (max-width: 700px) {
    .metrics-grid {
      grid-template-columns: 1fr;
    }
  }
  .small {
    font-size: 12px;
  }
  @media (max-width: 900px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
