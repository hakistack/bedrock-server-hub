<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { playersStore } from '$lib/stores/players.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { formatDate } from '$lib/util/format';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';

  let loadedFor = $state<string | null>(null);

  const server = $derived(serverStore.selected);
  const online = $derived(playersStore.onlineOf(serverStore.selectedId));
  const events = $derived(playersStore.eventsOf(serverStore.selectedId));

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) {
      loadedFor = id;
      api.getOnlinePlayers(id).then((p) => playersStore.setOnline(id, p)).catch(() => {});
    }
  });

  async function copyXuid(xuid: string) {
    try {
      await navigator.clipboard.writeText(xuid);
      toasts.success('XUID copiado.');
    } catch (err) {
      toasts.error(errorMessage(err));
    }
  }
</script>

<PageHeader title="Players" subtitle="Jugadores conectados y actividad (desde la consola del servidor).">
  {#snippet actions()}
    {#if server}<Badge tone={online.length ? 'success' : 'default'}>{online.length} online</Badge>{/if}
  {/snippet}
</PageHeader>

{#if !server}
  <div class="card"><EmptyState icon="👥" title="Sin servidor" description="Selecciona un servidor." /></div>
{:else}
  <div class="grid">
    <Card title={`Conectados (${online.length})`}>
      {#if online.length === 0}
        <p class="muted small">Nadie conectado ahora mismo.</p>
      {:else}
        {#each online as p (p.xuid)}
          <div class="player">
            <div class="avatar">{p.name.slice(0, 1).toUpperCase()}</div>
            <div class="p-info">
              <strong>{p.name}</strong>
              <div class="faint mono small">XUID {p.xuid} · desde {formatDate(p.connectedAt)}</div>
            </div>
            <Button size="sm" onclick={() => copyXuid(p.xuid)}>Copiar XUID</Button>
          </div>
        {/each}
      {/if}
    </Card>

    <Card title="Actividad reciente">
      {#if events.length === 0}
        <p class="muted small">Sin eventos todavía.</p>
      {:else}
        {#each events as e (e.at + e.xuid)}
          <div class="event">
            <span class="dot {e.event}"></span>
            <span><strong>{e.name}</strong> {e.event === 'connected' ? 'se conectó' : 'se desconectó'}</span>
            <span class="faint small when">{formatDate(e.at)}</span>
          </div>
        {/each}
      {/if}
    </Card>
  </div>

  <p class="faint small note">
    Salud, nivel, posición y armadura no los expone Bedrock por consola — requieren un pack de
    telemetría (próximo milestone).
  </p>
{/if}

<style>
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
    align-items: start;
  }
  .small {
    font-size: 12px;
  }
  .player {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
  }
  .player:last-child {
    border-bottom: none;
  }
  .avatar {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    color: var(--accent);
  }
  .p-info {
    flex: 1;
    min-width: 0;
  }
  .event {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    font-size: 13px;
  }
  .event:last-child {
    border-bottom: none;
  }
  .when {
    margin-left: auto;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .dot.connected {
    background: var(--accent);
  }
  .dot.disconnected {
    background: var(--text-faint);
  }
  .note {
    margin-top: 16px;
  }
  @media (max-width: 800px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
