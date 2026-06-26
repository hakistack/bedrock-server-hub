<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { page } from '$app/stores';

  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { NAV, isActive } from '$lib/config/nav';
  import Select from '$lib/components/shared/Select.svelte';
  import StatusBadge from '$lib/components/shared/StatusBadge.svelte';

  let appVersion = $state('');
  let busy = $state(false);

  const status = $derived(serverStore.selectedStatus);
  const canStart = $derived(status === 'offline' || status === 'crashed');
  const canStop = $derived(status === 'online' || status === 'starting');

  onMount(() => {
    getVersion()
      .then((v) => (appVersion = v))
      .catch(() => {});
  });

  async function quickStart() {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.startServer(id);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }
  async function quickStop() {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.stopServer(id);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }
</script>

<aside class="sidebar">
  <a class="brand" href="/">
    <img class="brand-logo" src="/favicon.png" alt="" />
    <span class="brand-name">Bedrock<br /><strong>Server Manager</strong></span>
  </a>

  <div class="picker">
    <span class="picker-label">Servidor activo</span>
    {#if serverStore.servers.length}
      <Select
        value={serverStore.selectedId}
        options={serverStore.servers.map((s) => ({ value: s.id, label: s.name }))}
        onChange={(v) => serverStore.select(v)}
        ariaLabel="Servidor seleccionado"
      />
      <div class="picker-foot">
        <StatusBadge {status} />
        <div class="quick">
          {#if canStop}
            <button class="q-btn stop" onclick={quickStop} disabled={busy} title="Detener">■</button>
          {:else}
            <button class="q-btn start" onclick={quickStart} disabled={busy || !canStart} title="Iniciar">▶</button>
          {/if}
        </div>
      </div>
    {:else}
      <p class="faint no-server">Sin servidores aún.</p>
    {/if}
  </div>

  <nav>
    {#each NAV as item (item.href)}
      <a class="nav-item" class:active={isActive($page.url.pathname, item.href)} href={item.href}>
        <span class="nav-icon">{item.icon}</span>
        <span>{item.label}</span>
      </a>
    {/each}
  </nav>

  <div class="footer">
    <a class="new-server" href="/new">+ Nuevo servidor</a>
    {#if appVersion}<span class="version mono">v{appVersion}</span>{/if}
  </div>
</aside>

<style>
  .sidebar {
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 18px 14px;
    height: 100%;
    overflow-y: auto;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 4px 6px 18px;
  }
  .brand-logo {
    width: 38px;
    height: 38px;
    border-radius: 9px;
    object-fit: cover;
    box-shadow: var(--shadow-sm);
  }
  .brand-name {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.25;
  }
  .brand-name strong {
    color: var(--text);
    font-size: 14px;
  }
  .picker {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px;
    margin-bottom: 18px;
  }
  .picker-label {
    display: block;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-faint);
    margin-bottom: 7px;
  }
  .picker-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 10px;
  }
  .quick {
    display: flex;
    gap: 6px;
  }
  .q-btn {
    width: 30px;
    height: 26px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    font-size: 11px;
  }
  .q-btn.start {
    color: var(--accent);
    border-color: var(--accent);
  }
  .q-btn.stop {
    color: #ff8a8a;
    border-color: var(--danger);
  }
  .q-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .no-server {
    margin: 4px 0 0;
    font-size: 13px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }
  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 9px 12px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-weight: 550;
    transition: background 0.12s, color 0.12s;
  }
  .nav-item:hover {
    background: var(--surface);
    color: var(--text);
  }
  .nav-item.active {
    background: var(--surface-2);
    color: var(--text);
  }
  .nav-item.active::before {
    content: '';
    position: absolute;
    left: -14px;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 20px;
    border-radius: 0 3px 3px 0;
    background: var(--accent);
  }
  .nav-icon {
    width: 20px;
    text-align: center;
  }
  .footer {
    padding-top: 14px;
    margin-top: 8px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .new-server {
    width: 100%;
    text-align: center;
    background: var(--accent);
    color: #04130d;
    font-weight: 650;
    padding: 9px 14px;
    border-radius: var(--radius-sm);
    transition: background 0.15s;
  }
  .new-server:hover {
    background: var(--accent-hover);
  }
  .version {
    font-size: 11px;
    color: var(--text-faint);
  }
</style>
