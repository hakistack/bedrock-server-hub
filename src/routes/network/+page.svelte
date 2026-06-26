<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import type { NetworkStatus } from '$lib/types/network';

  let status = $state<NetworkStatus | null>(null);
  let loading = $state(false);
  let busy = $state(false);
  let loadedFor = $state<string | null>(null);

  const server = $derived(serverStore.selected);
  const missingRules = $derived(status ? status.ports.filter((p) => !p.ruleExists).length : 0);

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
      toasts.success(`Puerto asignado: ${v4}. Reinicia el servidor para aplicarlo.`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }
</script>

<PageHeader title="Network" subtitle="Puertos del servidor y reglas de firewall entrantes (UDP).">
  {#snippet actions()}
    {#if server}
      <Button onclick={assignPort} loading={busy}>Asignar puerto libre</Button>
      {#if status?.firewallSupported}
        <Button variant="primary" onclick={addRules} loading={busy} disabled={missingRules === 0}>
          {missingRules > 0 ? `Agregar ${missingRules} regla(s)` : 'Reglas al día'}
        </Button>
      {/if}
    {/if}
  {/snippet}
</PageHeader>

{#if !server}
  <div class="card"><EmptyState icon="🛡️" title="Sin servidor" description="Selecciona un servidor para gestionar su red." /></div>
{:else if loading}
  <div class="card"><Spinner text="Comprobando puertos y firewall…" /></div>
{:else if status}
  {#if status.conflicts.length}
    <Card>
      <strong class="warn">⚠ Conflicto de puertos</strong>
      <p class="muted small">Este servidor comparte puerto con otro. Usa "Asignar puerto libre".</p>
      <ul class="conflicts">
        {#each status.conflicts as c (c.port + c.otherServer)}
          <li class="small">Puerto <span class="mono">{c.port}</span> también lo usa <strong>{c.otherServer}</strong></li>
        {/each}
      </ul>
    </Card>
    <div style="height:16px;"></div>
  {/if}

  <div class="ports">
    {#each status.ports as p (p.key)}
      <div class="port">
        <div class="port-info">
          <div class="row" style="gap:10px;">
            <Badge tone="info">{p.protocol}</Badge>
            <h3 class="mono">{p.port}</h3>
            <span class="faint">{p.label}</span>
          </div>
          <p class="faint mono small">{p.key} · {p.ruleName}</p>
        </div>
        {#if !status.firewallSupported}
          <Badge>Firewall: solo Windows</Badge>
        {:else if p.ruleExists}
          <Badge tone="success">✓ En el firewall</Badge>
        {:else}
          <Badge tone="danger">✗ Sin regla</Badge>
        {/if}
      </div>
    {/each}
  </div>

  <Card>
    {#if status.firewallSupported}
      <p class="muted small">Crear reglas requiere permisos de administrador: Windows mostrará un aviso de <strong>UAC</strong>. Se crean reglas <span class="mono">UDP</span> entrantes.</p>
    {:else}
      <p class="muted small">La gestión automática del firewall es solo <strong>Windows</strong> (plataforma actual: <span class="mono">{status.platform}</span>). Aquí puedes ver y reasignar puertos; ábrelos manualmente en tu firewall.</p>
    {/if}
  </Card>
{/if}

<style>
  .warn {
    color: var(--warning);
  }
  .conflicts {
    margin: 8px 0 0;
    padding-left: 18px;
  }
  .small {
    font-size: 12px;
  }
  .ports {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 16px;
  }
  .port {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px 18px;
  }
  .port h3 {
    font-size: 20px;
  }
  .port-info .small {
    margin: 6px 0 0;
  }
</style>
