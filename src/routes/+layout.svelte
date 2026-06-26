<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { afterNavigate } from '$app/navigation';

  import '$lib/styles/app.css';
  import { onServerLog, onServerStatus, onServerMetrics, onPlayerEvent } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { logsStore } from '$lib/stores/logs.store.svelte';
  import { metricsStore } from '$lib/stores/metrics.store.svelte';
  import { playersStore } from '$lib/stores/players.store.svelte';
  import StatusBadge from '$lib/components/shared/StatusBadge.svelte';
  import Select from '$lib/components/shared/Select.svelte';
  import Toasts from '$lib/components/shared/Toasts.svelte';
  import UpdateBanner from '$lib/components/shared/UpdateBanner.svelte';

  let { children } = $props();

  type NavItem = { href: string; label: string; icon: string; soon?: boolean };
  const nav: NavItem[] = [
    { href: '/', label: 'Dashboard', icon: '🏠' },
    { href: '/console', label: 'Console', icon: '🖥️' },
    { href: '/players', label: 'Players', icon: '👥' },
    { href: '/settings', label: 'Settings', icon: '⚙️' },
    { href: '/network', label: 'Red / Firewall', icon: '🛡️' },
    { href: '/worlds', label: 'Worlds', icon: '🌍' },
    { href: '/addons', label: 'Addons', icon: '🧩' },
    { href: '/backups', label: 'Backups', icon: '💾' },
  ];

  onMount(() => {
    serverStore.refresh();

    const unlisteners: Array<() => void> = [];
    onServerStatus((evt) => serverStore.setStatus(evt.serverId, evt.status)).then((un) =>
      unlisteners.push(un),
    );
    onServerLog((log) => logsStore.append(log)).then((un) => unlisteners.push(un));
    onServerMetrics((m) => metricsStore.append(m)).then((un) => unlisteners.push(un));
    onPlayerEvent((e) => playersStore.applyEvent(e)).then((un) => unlisteners.push(un));

    return () => unlisteners.forEach((un) => un());
  });

  function isActive(href: string): boolean {
    const path = $page.url.pathname;
    return href === '/' ? path === '/' : path.startsWith(href);
  }

  // --- Back / forward navigation (browser-style, in-app history) ---
  let navIdx = $state(0);
  let navMax = $state(0);
  const canBack = $derived(navIdx > 0);
  const canForward = $derived(navIdx < navMax);

  afterNavigate((nav) => {
    if (nav.type === 'enter') {
      navIdx = 0;
      navMax = 0;
    } else if (nav.type === 'popstate') {
      const delta = (nav as { delta?: number }).delta ?? 0;
      navIdx = Math.max(0, Math.min(navMax, navIdx + delta));
    } else {
      // A fresh push truncates any forward history.
      navIdx += 1;
      navMax = navIdx;
    }
  });

  const goBack = () => history.back();
  const goForward = () => history.forward();

  const currentLabel = $derived(
    nav.find((n) => isActive(n.href))?.label ?? '',
  );
</script>

<div class="app">
  <aside class="sidebar">
    <div class="brand">
      <img class="brand-logo" src="/favicon.png" alt="" />
      <span class="brand-name">Bedrock<br /><strong>Server Manager</strong></span>
    </div>

    <div class="server-picker">
      <span class="picker-label">Servidor</span>
      {#if serverStore.servers.length}
        <Select
          value={serverStore.selectedId}
          options={serverStore.servers.map((s) => ({ value: s.id, label: s.name }))}
          onChange={(v) => serverStore.select(v)}
          ariaLabel="Servidor seleccionado"
        />
        <div class="picker-status">
          <StatusBadge status={serverStore.selectedStatus} />
        </div>
      {:else}
        <p class="faint no-server">Ningún servidor registrado.</p>
      {/if}
    </div>

    <nav>
      {#each nav as item (item.href)}
        <a class="nav-item" class:active={isActive(item.href)} href={item.href}>
          <span class="nav-icon">{item.icon}</span>
          <span>{item.label}</span>
          {#if item.soon}<span class="soon">pronto</span>{/if}
        </a>
      {/each}
    </nav>

    <div class="sidebar-footer">
      <a class="btn btn-primary new-server" href="/new">+ Nuevo servidor</a>
    </div>
  </aside>

  <main class="content">
    <div class="topbar">
      <div class="nav-arrows">
        <button class="arrow" onclick={goBack} disabled={!canBack} aria-label="Atrás" title="Atrás">
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path d="M10 3l-5 5 5 5" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <button class="arrow" onclick={goForward} disabled={!canForward} aria-label="Adelante" title="Adelante">
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </div>
      {#if currentLabel}<span class="crumb">{currentLabel}</span>{/if}
      <div class="topbar-right">
        <UpdateBanner />
      </div>
    </div>
    <div class="page">
      {@render children()}
    </div>
  </main>
</div>

<Toasts />

<style>
  .app {
    display: grid;
    grid-template-columns: 248px 1fr;
    height: 100vh;
    overflow: hidden;
  }
  .sidebar {
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 18px 14px;
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
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
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
  .server-picker {
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
  .picker-status {
    margin-top: 10px;
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
    font-weight: 500;
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
  .soon {
    margin-left: auto;
    font-size: 10px;
    text-transform: uppercase;
    color: var(--text-faint);
    border: 1px solid var(--border);
    padding: 1px 6px;
    border-radius: 999px;
  }
  .sidebar-footer {
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }
  .new-server {
    width: 100%;
    justify-content: center;
    text-align: center;
  }
  .content {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }
  .topbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 32px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .nav-arrows {
    display: flex;
    gap: 6px;
  }
  .arrow {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    transition: background 0.12s, color 0.12s;
  }
  .arrow:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text);
  }
  .arrow:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .crumb {
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 500;
  }
  .topbar-right {
    margin-left: auto;
  }
  .page {
    flex: 1;
    overflow-y: auto;
    padding: 28px 32px;
  }
</style>
