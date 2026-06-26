<script lang="ts">
  import { onMount } from 'svelte';
  import { api, onUpdateProgress } from '$lib/api/commands';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { humanSize } from '$lib/util/format';
  import type { UpdateInfo, UpdateProgress } from '$lib/types/update';

  let info = $state<UpdateInfo | null>(null);
  let open = $state(false);
  let installing = $state(false);
  let progress = $state<UpdateProgress | null>(null);

  const available = $derived(info?.available ?? false);
  const pct = $derived(progress?.percentage != null ? Math.round(progress.percentage) : null);

  onMount(() => {
    // Best-effort check; network failures are silent.
    api
      .checkForUpdate()
      .then((i) => (info = i))
      .catch(() => {});

    let unlisten: (() => void) | null = null;
    onUpdateProgress((p) => (progress = p)).then((un) => (unlisten = un));
    return () => unlisten?.();
  });

  async function install() {
    if (!info?.downloadUrl || installing) return;
    installing = true;
    progress = null;
    try {
      // On success the app relaunches/exits, so this usually does not resolve.
      await api.downloadAndInstallUpdate(info.downloadUrl);
    } catch (err) {
      installing = false;
      toasts.error(errorMessage(err));
    }
  }
</script>

{#if available}
  <button class="update-pill" onclick={() => (open = true)} title="Actualización disponible">
    <span class="dot"></span>
    Actualizar a v{info?.latestVersion}
  </button>
{/if}

{#if open && info}
  <div
    class="overlay"
    role="button"
    tabindex="0"
    onclick={() => !installing && (open = false)}
    onkeydown={(e) => e.key === 'Escape' && !installing && (open = false)}
  >
    <div
      class="modal card"
      role="dialog"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={() => {}}
    >
      <h2>Actualización disponible</h2>
      <p class="muted small">
        v{info.currentVersion} → <strong>v{info.latestVersion}</strong>
        {#if info.assetSize}· {humanSize(info.assetSize)}{/if}
      </p>

      {#if info.notes}
        <div class="notes">{info.notes}</div>
      {/if}

      {#if installing}
        <div class="bar">
          <div
            class="bar-fill"
            style:width={`${pct ?? 0}%`}
            class:indeterminate={pct == null}
          ></div>
        </div>
        <p class="muted small center">
          {pct != null ? `Descargando… ${pct}%` : 'Preparando…'} — la app se reiniciará al terminar.
        </p>
      {:else}
        <div class="modal-actions">
          <button class="btn" onclick={() => (open = false)}>Después</button>
          <button class="btn btn-primary" onclick={install}>Actualizar ahora</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .update-pill {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    background: rgba(59, 165, 93, 0.15);
    border: 1px solid var(--accent);
    color: var(--accent);
    padding: 5px 11px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
  }
  .update-pill .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 600;
  }
  .modal {
    width: 480px;
    max-width: 90vw;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .small {
    font-size: 12px;
  }
  .center {
    text-align: center;
  }
  .notes {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px;
    max-height: 240px;
    overflow-y: auto;
    white-space: pre-wrap;
    font-size: 13px;
    color: var(--text-muted);
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
  .bar {
    height: 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 999px;
    overflow: hidden;
  }
  .bar-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.2s ease;
  }
  .bar-fill.indeterminate {
    width: 40% !important;
    animation: slide 1.2s infinite ease-in-out;
  }
  @keyframes slide {
    0% {
      margin-left: -40%;
    }
    100% {
      margin-left: 100%;
    }
  }
</style>
