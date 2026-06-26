<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { playersStore } from '$lib/stores/players.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { formatDate } from '$lib/util/format';

  let loadedFor = $state<string | null>(null);

  const server = $derived(serverStore.selected);
  const online = $derived(playersStore.onlineOf(serverStore.selectedId));
  const events = $derived(playersStore.eventsOf(serverStore.selectedId));

  // Seed the online list from the backend (covers players connected before
  // this view, or events missed while elsewhere).
  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) {
      loadedFor = id;
      api
        .getOnlinePlayers(id)
        .then((p) => playersStore.setOnline(id, p))
        .catch(() => {});
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

  function relTime(iso: string): string {
    return formatDate(iso);
  }
</script>

<header class="page-head row spread">
  <div>
    <h1>Players</h1>
    <p class="muted">Jugadores conectados y actividad (desde la consola del servidor).</p>
  </div>
  {#if server}<span class="count">{online.length} online</span>{/if}
</header>

{#if !server}
  <div class="card empty-state">Selecciona o importa un servidor.</div>
{:else}
  <div class="grid">
    <section class="card">
      <div class="card-title">Conectados ({online.length})</div>
      {#if online.length === 0}
        <p class="muted small">Nadie conectado ahora mismo.</p>
      {:else}
        {#each online as p (p.xuid)}
          <div class="player-row">
            <div class="avatar">{p.name.slice(0, 1).toUpperCase()}</div>
            <div class="p-info">
              <strong>{p.name}</strong>
              <div class="faint mono small">XUID {p.xuid} · desde {relTime(p.connectedAt)}</div>
            </div>
            <button class="btn btn-sm" onclick={() => copyXuid(p.xuid)}>Copiar XUID</button>
          </div>
        {/each}
      {/if}
    </section>

    <section class="card">
      <div class="card-title">Actividad reciente</div>
      {#if events.length === 0}
        <p class="muted small">Sin eventos todavía.</p>
      {:else}
        {#each events as e (e.at + e.xuid)}
          <div class="event-row">
            <span class="dot {e.event}"></span>
            <span>
              <strong>{e.name}</strong>
              {e.event === 'connected' ? 'se conectó' : 'se desconectó'}
            </span>
            <span class="faint small when">{relTime(e.at)}</span>
          </div>
        {/each}
      {/if}
    </section>
  </div>

  <p class="faint small note">
    Nota: salud, nivel, posición y armadura no los expone Bedrock por consola — requieren un pack de
    telemetría (próximo milestone). Aquí se muestran online/XUID/conexiones a partir del log.
  </p>
{/if}

<style>
  .page-head {
    margin-bottom: 22px;
    align-items: flex-start;
  }
  .count {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: 999px;
    padding: 4px 12px;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
    align-items: start;
  }
  .small {
    font-size: 12px;
  }
  .player-row {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
  }
  .player-row:last-child {
    border-bottom: none;
  }
  .avatar {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    background: var(--surface-2);
    border: 1px solid var(--border);
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
  .event-row {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    font-size: 13px;
  }
  .event-row:last-child {
    border-bottom: none;
  }
  .event-row .when {
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
