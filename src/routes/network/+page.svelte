<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import type { NetworkStatus } from '$lib/types/network';

  let status = $state<NetworkStatus | null>(null);
  let loading = $state(false);
  let busy = $state(false);
  let loadedFor = $state<string | null>(null);

  const server = $derived(serverStore.selected);
  const missingRules = $derived(
    status ? status.ports.filter((p) => !p.ruleExists).length : 0,
  );

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) load(id);
  });

  async function load(id: string) {
    loading = true;
    try {
      status = await api.getNetworkStatus(id);
      loadedFor = id;
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      loading = false;
    }
  }

  async function addRules() {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      status = await api.addFirewallRules(id);
      toasts.success('Reglas de firewall verificadas/creadas.');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }

  async function assignPort() {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      status = await api.assignFreePort(id);
      const v4 = status.ports.find((p) => p.label === 'IPv4')?.port;
      toasts.success(`Puerto asignado: ${v4} (IPv4). Reinicia el servidor para aplicarlo.`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }
</script>

<header class="page-head row spread">
  <div>
    <h1>Red / Firewall</h1>
    <p class="muted">Puertos del servidor y reglas de firewall entrantes (UDP).</p>
  </div>
  {#if server}
    <div class="row" style="gap:10px;">
      <button class="btn" onclick={assignPort} disabled={busy}>Asignar puerto libre</button>
      {#if status?.firewallSupported}
        <button class="btn btn-primary" onclick={addRules} disabled={busy || missingRules === 0}>
          {missingRules > 0 ? `Agregar ${missingRules} regla(s)` : 'Reglas al día'}
        </button>
      {/if}
    </div>
  {/if}
</header>

{#if !server}
  <div class="card empty-state">Selecciona o importa un servidor para gestionar su red.</div>
{:else if loading}
  <div class="card muted">Comprobando puertos y firewall…</div>
{:else if status}
  {#if status.conflicts.length}
    <div class="card conflict">
      <strong>⚠ Conflicto de puertos</strong>
      <p class="muted small">
        Este servidor comparte puerto con otro. Dos servidores Bedrock no pueden usar el mismo
        puerto a la vez — usa “Asignar puerto libre”.
      </p>
      <ul>
        {#each status.conflicts as c (c.port + c.otherServer)}
          <li class="small">Puerto <span class="mono">{c.port}</span> también lo usa <strong>{c.otherServer}</strong></li>
        {/each}
      </ul>
    </div>
  {/if}

  <div class="ports">
    {#each status.ports as p (p.key)}
      <div class="card port">
        <div class="port-info">
          <div class="row" style="gap:10px;">
            <span class="proto">{p.protocol}</span>
            <h3>{p.port}</h3>
            <span class="faint">{p.label}</span>
          </div>
          <p class="faint mono small">{p.key} · regla: {p.ruleName}</p>
        </div>
        <div class="fw-status">
          {#if !status.firewallSupported}
            <span class="badge na">Firewall: solo Windows</span>
          {:else if p.ruleExists}
            <span class="badge ok">✓ En el firewall</span>
          {:else}
            <span class="badge missing">✗ Sin regla</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <div class="card note">
    {#if status.firewallSupported}
      <p class="muted small">
        Crear reglas requiere permisos de administrador: Windows mostrará un aviso de
        <strong>UAC</strong> al pulsar “Agregar reglas”. Se crean reglas <span class="mono">UDP</span>
        entrantes para los puertos de este servidor.
      </p>
    {:else}
      <p class="muted small">
        La gestión automática del firewall está disponible solo en <strong>Windows</strong>
        (plataforma actual: <span class="mono">{status.platform}</span>). Aquí puedes igualmente ver
        y reasignar los puertos; ábrelos manualmente en el firewall de tu sistema.
      </p>
    {/if}
  </div>
{/if}

<style>
  .page-head {
    margin-bottom: 22px;
    align-items: flex-start;
  }
  .small {
    font-size: 12px;
  }
  .conflict {
    border-color: var(--warning);
    margin-bottom: 16px;
  }
  .conflict strong {
    color: var(--warning);
  }
  .conflict ul {
    margin: 8px 0 0;
    padding-left: 18px;
  }
  .ports {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .port {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .proto {
    font-size: 11px;
    font-weight: 700;
    color: var(--info);
    border: 1px solid var(--info);
    border-radius: 6px;
    padding: 2px 7px;
  }
  .port h3 {
    font-size: 20px;
    font-family: ui-monospace, monospace;
  }
  .badge {
    font-size: 12px;
    font-weight: 600;
    padding: 5px 11px;
    border-radius: 999px;
    border: 1px solid var(--border);
  }
  .badge.ok {
    color: var(--accent);
    border-color: var(--accent);
  }
  .badge.missing {
    color: var(--danger);
    border-color: var(--danger);
  }
  .badge.na {
    color: var(--text-muted);
  }
  .note {
    margin-top: 16px;
  }
</style>
