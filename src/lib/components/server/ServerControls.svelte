<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';

  let busy = $state(false);

  const status = $derived(serverStore.selectedStatus);
  const canStart = $derived(status === 'offline' || status === 'crashed');
  const canStop = $derived(status === 'online' || status === 'starting');

  async function run(action: () => Promise<void>) {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await action();
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }

  const start = () => run(async () => { await api.startServer(serverStore.selectedId!); });
  const stop = () => run(async () => { await api.stopServer(serverStore.selectedId!); });
  const restart = () => run(async () => { await api.restartServer(serverStore.selectedId!); });
</script>

<div class="controls">
  <button class="btn btn-primary" onclick={start} disabled={busy || !canStart}>▶ Start</button>
  <button class="btn" onclick={stop} disabled={busy || !canStop}>■ Stop</button>
  <button class="btn" onclick={restart} disabled={busy || !canStop}>↻ Restart</button>
</div>

<style>
  .controls {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
</style>
