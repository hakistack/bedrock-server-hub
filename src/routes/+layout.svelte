<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';

  import '$lib/styles/app.css';
  import { onServerLog, onServerStatus, onServerMetrics, onPlayerEvent } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { logsStore } from '$lib/stores/logs.store.svelte';
  import { metricsStore } from '$lib/stores/metrics.store.svelte';
  import { playersStore } from '$lib/stores/players.store.svelte';
  import Sidebar from '$lib/components/shell/Sidebar.svelte';
  import TopBar from '$lib/components/shell/TopBar.svelte';
  import Toasts from '$lib/components/shared/Toasts.svelte';

  let { children } = $props();

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
</script>

<div class="app">
  <Sidebar />
  <main class="content">
    <TopBar />
    <div class="page">
      {#key $page.url.pathname}
        <div class="page-anim">{@render children()}</div>
      {/key}
    </div>
  </main>
</div>

<Toasts />

<style>
  .app {
    display: grid;
    grid-template-columns: 256px 1fr;
    height: 100vh;
    overflow: hidden;
  }
  .content {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }
  .page {
    flex: 1;
    overflow-y: auto;
    padding: 28px 32px;
  }
</style>
