<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { logsStore } from '$lib/stores/logs.store.svelte';
  import { metricsStore } from '$lib/stores/metrics.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { humanSize, formatDate } from '$lib/util/format';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import StatChip from '$lib/components/ui/StatChip.svelte';
  import StatusBadge from '$lib/components/shared/StatusBadge.svelte';
  import ServerControls from '$lib/components/server/ServerControls.svelte';
  import Sparkline from '$lib/components/shared/Sparkline.svelte';
  import LogView from '$lib/components/shared/LogView.svelte';

  const server = $derived(serverStore.selected);
  const status = $derived(serverStore.selectedStatus);
  const recentLogs = $derived(logsStore.get(serverStore.selectedId).slice(-10));
  const metrics = $derived(metricsStore.get(serverStore.selectedId));
  const latest = $derived(metrics.at(-1));

  const statusLabel: Record<string, string> = {
    offline: 'Offline',
    starting: 'Starting',
    online: 'Online',
    stopping: 'Stopping',
    crashed: 'Crashed',
  };

  // Per-server overview data.
  let activeWorld = $state('—');
  let addonCount = $state(0);
  let lastBackup = $state<string | null>(null);
  let autoRestart = $state(false);
  let loadedFor = $state<string | null>(null);
  let busy = $state(false);

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) {
      loadedFor = id;
      loadOverview(id);
    }
  });

  async function loadOverview(id: string) {
    try {
      const props = await api.readProperties(id);
      activeWorld = props.find((p) => p.key === 'level-name')?.value || 'Bedrock level';
      const installed = await api.listInstalledAddons(id);
      addonCount = installed.length;
      const backups = await api.listBackups(id);
      lastBackup = backups[0]?.createdAt ?? null;
      const settings = await api.getServerSettings(id);
      autoRestart = settings.autoRestart;
    } catch {
      /* best-effort */
    }
  }

  async function createBackup() {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.createBackup(id);
      const backups = await api.listBackups(id);
      lastBackup = backups[0]?.createdAt ?? null;
      toasts.success('Backup creado.');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }

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

{#if serverStore.loaded && serverStore.servers.length === 0}
  <!-- Onboarding -->
  <div class="onboarding">
    <div class="hero">
      <img class="logo" src="/favicon.png" alt="" />
      <h1>Bienvenido a Bedrock Server Manager</h1>
      <p class="muted">
        Crea o importa tu primer servidor dedicado de Minecraft Bedrock. Gestiona mundos, addons,
        backups y más — sin tocar archivos a mano.
      </p>
    </div>
    <div class="choices">
      <a class="choice" href="/new">
        <div class="ch-icon">⬇️</div>
        <h2>Descargar servidor oficial</h2>
        <p class="muted">Descarga y prepara un servidor nuevo desde la web oficial de Minecraft.</p>
      </a>
      <a class="choice" href="/new">
        <div class="ch-icon">📂</div>
        <h2>Importar servidor existente</h2>
        <p class="muted">Usa una carpeta de Bedrock Dedicated Server que ya tienes.</p>
      </a>
    </div>
  </div>
{:else}
  <PageHeader title="Dashboard" subtitle="Estado y control de tu servidor.">
    {#snippet actions()}
      {#if server}<StatusBadge {status} />{/if}
    {/snippet}
  </PageHeader>

  {#if !server}
    <Card>
      <EmptyState
        icon="🗄️"
        title="Ningún servidor seleccionado"
        description="Elige un servidor en la barra lateral, o crea/importa uno nuevo."
      >
        {#snippet actions()}
          <Button variant="primary" href="/new">+ Nuevo servidor</Button>
        {/snippet}
      </EmptyState>
    </Card>
  {:else}
    <div class="stats">
      <StatChip
        label="Estado"
        value={statusLabel[status]}
        icon="🟢"
        tone={status === 'online' ? 'success' : status === 'crashed' ? 'danger' : 'default'}
      />
      <StatChip label="Mundo activo" value={activeWorld} icon="🌍" />
      <StatChip label="Addons" value={String(addonCount)} icon="🧩" />
      <StatChip
        label="Último backup"
        value={lastBackup ? formatDate(lastBackup) : 'Nunca'}
        icon="💾"
      />
    </div>

    <div class="grid">
      <Card title="Control del servidor">
        <p class="faint mono path">{server.path}</p>
        <div class="controls"><ServerControls /></div>
        <label class="auto-restart">
          <input type="checkbox" checked={autoRestart} onchange={toggleAutoRestart} />
          <span>Reiniciar automáticamente tras un crash</span>
        </label>
      </Card>

      <Card title="Acciones rápidas">
        <div class="quick">
          <Button onclick={() => goto('/addons')}>🧩 Instalar addon</Button>
          <Button onclick={() => goto('/worlds')}>🌍 Importar mundo</Button>
          <Button onclick={createBackup} loading={busy}>💾 Crear backup</Button>
          <Button onclick={() => goto('/console')}>🖥️ Abrir consola</Button>
        </div>
      </Card>
    </div>

    <Card title="Rendimiento del proceso">
      {#if metrics.length === 0}
        <p class="muted">Sin datos. Inicia el servidor para ver CPU y memoria en vivo.</p>
      {:else}
        <div class="metrics">
          <div>
            <div class="m-head">
              <span class="muted">CPU</span><span class="m-val">{latest ? latest.cpu.toFixed(0) : 0}%</span>
            </div>
            <Sparkline values={metrics.map((m) => m.cpu)} color="#6fb1ff" />
          </div>
          <div>
            <div class="m-head">
              <span class="muted">Memoria</span>
              <span class="m-val">{latest ? humanSize(latest.memoryBytes) : '—'}</span>
            </div>
            <Sparkline values={metrics.map((m) => m.memoryBytes)} color="var(--accent)" />
          </div>
        </div>
      {/if}
    </Card>

    <Card>
      {#snippet actions()}
        <a class="link" href="/console">Ver consola completa →</a>
      {/snippet}
      {#snippet children()}
        <div class="card-title" style="margin:0 0 12px;">Últimos logs</div>
        <LogView lines={recentLogs} height="240px" />
      {/snippet}
    </Card>
  {/if}
{/if}

<style>
  .onboarding {
    max-width: 760px;
    margin: 4vh auto 0;
    text-align: center;
  }
  .hero .logo {
    width: 72px;
    height: 72px;
    border-radius: 16px;
    box-shadow: var(--shadow);
  }
  .hero h1 {
    font-size: 28px;
    margin: 18px 0 10px;
  }
  .hero .muted {
    max-width: 540px;
    margin: 0 auto;
  }
  .choices {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-top: 32px;
  }
  .choice {
    text-align: left;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 22px;
    transition: border-color 0.15s, transform 0.1s;
  }
  .choice:hover {
    border-color: var(--accent);
    transform: translateY(-2px);
  }
  .ch-icon {
    font-size: 30px;
    margin-bottom: 10px;
  }
  .choice h2 {
    font-size: 17px;
    margin-bottom: 8px;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 14px;
    margin-bottom: 18px;
  }
  .grid {
    display: grid;
    grid-template-columns: 1.4fr 1fr;
    gap: 18px;
    margin-bottom: 18px;
  }
  .path {
    margin: 0 0 16px;
    font-size: 12px;
    word-break: break-all;
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
  .quick {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .metrics {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
  }
  .m-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 7px;
  }
  .m-val {
    font-family: ui-monospace, monospace;
    font-weight: 650;
  }
  .link {
    color: var(--text-muted);
    font-size: 12px;
  }
  .link:hover {
    color: var(--accent);
  }
  @media (max-width: 1100px) {
    .stats {
      grid-template-columns: repeat(2, 1fr);
    }
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
