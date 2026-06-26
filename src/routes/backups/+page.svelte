<script lang="ts">
  import { onMount } from 'svelte';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { api, onBackupProgress } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { formatDate, backupReasonLabel } from '$lib/util/format';
  import Select from '$lib/components/shared/Select.svelte';
  import NumberField from '$lib/components/shared/NumberField.svelte';
  import Toggle from '$lib/components/shared/Toggle.svelte';
  import type { BackupProgress, BackupRecord, BackupSchedule } from '$lib/types/backup';

  let backups = $state<BackupRecord[]>([]);
  let loading = $state(false);
  let busy = $state(false);
  let loadedFor = $state<string | null>(null);

  // Automated backup schedule.
  let schedule = $state<BackupSchedule | null>(null);
  let intervalStr = $state('60');
  let retentionStr = $state('7');
  let savingSchedule = $state(false);

  // Restore options panel.
  let restoring = $state<BackupRecord | null>(null);
  let restoreWorld = $state(true);
  let restoreProperties = $state(true);

  // Live zip progress.
  let progress = $state<BackupProgress | null>(null);

  const server = $derived(serverStore.selected);

  onMount(() => {
    let unlisten: (() => void) | null = null;
    onBackupProgress((p) => {
      if (p.phase === 'completed' || p.phase === 'starting') {
        // Brief, no-op for tiny backups; only the zipping phase shows a bar.
        if (p.phase === 'completed') progress = null;
      } else {
        progress = p;
      }
    }).then((un) => (unlisten = un));
    return () => unlisten?.();
  });

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) load(id);
  });

  async function load(id: string) {
    loading = true;
    try {
      backups = await api.listBackups(id);
      schedule = await api.getBackupSchedule(id);
      intervalStr = String(schedule.intervalMinutes);
      retentionStr = String(schedule.retention);
      loadedFor = id;
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      loading = false;
    }
  }

  async function saveSchedule() {
    const id = serverStore.selectedId;
    if (!id || !schedule || savingSchedule) return;
    schedule.intervalMinutes = Math.max(5, parseInt(intervalStr, 10) || 60);
    schedule.retention = Math.max(0, parseInt(retentionStr, 10) || 0);
    savingSchedule = true;
    try {
      await api.setBackupSchedule(id, schedule);
      toasts.success('Programación de backups guardada.');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      savingSchedule = false;
    }
  }

  async function reload() {
    if (serverStore.selectedId) await load(serverStore.selectedId);
  }

  async function createManual() {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.createBackup(id);
      await reload();
      toasts.success('Backup manual creado (server.properties).');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
      progress = null;
    }
  }

  function openRestore(b: BackupRecord) {
    restoring = b;
    restoreWorld = !!b.worldName;
    restoreProperties = true;
  }

  async function confirmRestore() {
    const b = restoring;
    if (!b || busy) return;
    if (!restoreWorld && !restoreProperties) {
      toasts.error('Selecciona al menos un elemento a restaurar.');
      return;
    }
    busy = true;
    try {
      await api.restoreBackup(b.id, { restoreWorld, restoreProperties });
      restoring = null;
      await reload();
      toasts.success('Backup restaurado. Reinicia el servidor para aplicar los cambios.');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
      progress = null;
    }
  }

  async function remove(b: BackupRecord) {
    if (busy) return;
    const ok = await confirm('¿Eliminar este backup de forma permanente?', {
      title: 'Eliminar backup',
      kind: 'warning',
    });
    if (!ok) return;
    busy = true;
    try {
      await api.deleteBackup(b.id);
      await reload();
      toasts.success('Backup eliminado.');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }

  const pct = $derived(
    progress && progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : null,
  );
</script>

<header class="page-head row spread">
  <div>
    <h1>Backups</h1>
    <p class="muted">Copias de seguridad del servidor y sus mundos.</p>
  </div>
  {#if server}
    <button class="btn btn-primary" onclick={createManual} disabled={busy}>
      + Backup manual
    </button>
  {/if}
</header>

{#if progress}
  <div class="card progress-card">
    <div class="row spread">
      <span>Comprimiendo mundo…</span>
      <span class="mono faint">{progress.done}/{progress.total}{pct != null ? ` · ${pct}%` : ''}</span>
    </div>
    <div class="bar"><div class="bar-fill" style:width={`${pct ?? 30}%`}></div></div>
  </div>
{/if}

{#if server && schedule}
  <div class="card schedule-card">
    <div class="row spread">
      <div class="card-title" style="margin:0;">Backups automáticos</div>
      <Toggle
        checked={schedule.enabled}
        onToggle={(v) => (schedule!.enabled = v)}
        onLabel="Activado"
        offLabel="Desactivado"
      />
    </div>
    {#if schedule.enabled}
      <div class="sched-grid">
        <div class="field">
          <span class="field-label">Modo</span>
          <Select
            bind:value={schedule.mode}
            options={[
              { value: 'interval', label: 'Por intervalo' },
              { value: 'daily', label: 'Diario (hora fija)' },
            ]}
            ariaLabel="Modo de backup"
          />
        </div>
        {#if schedule.mode === 'daily'}
          <div class="field">
            <span class="field-label">Hora (HH:MM)</span>
            <input class="input mono" type="time" bind:value={schedule.dailyTime} />
          </div>
        {:else}
          <div class="field">
            <span class="field-label">Cada (minutos)</span>
            <NumberField bind:value={intervalStr} min={5} step={5} ariaLabel="Intervalo en minutos" />
          </div>
        {/if}
        <div class="field">
          <span class="field-label">Retención (máx. backups)</span>
          <NumberField bind:value={retentionStr} min={0} ariaLabel="Retención" />
        </div>
      </div>
      <p class="faint small note">
        Hace backup del mundo activo y conserva los últimos N. Requiere la app abierta para disparar.
      </p>
    {/if}
    <div class="row" style="justify-content:flex-end; margin-top:6px;">
      <button class="btn btn-primary" onclick={saveSchedule} disabled={savingSchedule}>
        {savingSchedule ? 'Guardando…' : 'Guardar programación'}
      </button>
    </div>
  </div>
{/if}

{#if !server}
  <div class="card empty-state">Selecciona o importa un servidor para ver sus backups.</div>
{:else if loading}
  <div class="card muted">Cargando backups…</div>
{:else if backups.length === 0}
  <div class="card empty-state">
    <h2>💾 Sin backups todavía</h2>
    <p class="muted">
      Se crean automáticamente antes de operaciones que modifican mundos o configuración, o
      manualmente desde aquí.
    </p>
  </div>
{:else}
  <div class="table">
    <div class="thead">
      <span>Fecha</span>
      <span>Razón</span>
      <span>Mundo</span>
      <span></span>
    </div>
    {#each backups as b (b.id)}
      <div class="trow">
        <span>{formatDate(b.createdAt)}</span>
        <span><span class="reason">{backupReasonLabel(b.reason)}</span></span>
        <span class="mono faint">{b.worldName ?? '—'}</span>
        <span class="actions">
          <button class="btn btn-sm" onclick={() => openRestore(b)} disabled={busy}>↺ Restaurar</button>
          <button class="btn btn-sm btn-danger" onclick={() => remove(b)} disabled={busy}>
            Eliminar
          </button>
        </span>
      </div>
    {/each}
  </div>
{/if}

<!-- Restore options modal -->
{#if restoring}
  <div
    class="overlay"
    role="button"
    tabindex="0"
    onclick={() => (restoring = null)}
    onkeydown={(e) => e.key === 'Escape' && (restoring = null)}
  >
    <div class="modal card" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <h2>Restaurar backup</h2>
      <p class="muted small">
        {backupReasonLabel(restoring.reason)} · {formatDate(restoring.createdAt)}
      </p>
      <p class="faint small">
        Se creará un backup de seguridad del estado actual antes de restaurar.
      </p>
      <div class="opts">
        <label class="opt">
          <input type="checkbox" bind:checked={restoreWorld} disabled={!restoring.worldName} />
          Restaurar mundo {restoring.worldName ? `("${restoring.worldName}")` : '(no incluido en este backup)'}
        </label>
        <label class="opt">
          <input type="checkbox" bind:checked={restoreProperties} />
          Restaurar server.properties
        </label>
      </div>
      <div class="modal-actions">
        <button class="btn" onclick={() => (restoring = null)} disabled={busy}>Cancelar</button>
        <button class="btn btn-primary" onclick={confirmRestore} disabled={busy}>
          {busy ? 'Restaurando…' : 'Restaurar'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page-head {
    margin-bottom: 22px;
    align-items: flex-start;
  }
  .progress-card {
    margin-bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .schedule-card {
    margin-bottom: 16px;
  }
  .sched-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
    margin: 14px 0 4px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .field-label {
    font-size: 13px;
    font-weight: 500;
  }
  .note {
    margin: 4px 0 0;
  }
  @media (max-width: 760px) {
    .sched-grid {
      grid-template-columns: 1fr;
    }
  }
  .bar {
    height: 10px;
    background: var(--bg-elevated);
    border-radius: 999px;
    overflow: hidden;
    border: 1px solid var(--border);
  }
  .bar-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.2s ease;
  }
  .table {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }
  .thead,
  .trow {
    display: grid;
    grid-template-columns: 1.3fr 1.3fr 1fr auto;
    gap: 12px;
    align-items: center;
    padding: 12px 16px;
  }
  .thead {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-faint);
    border-bottom: 1px solid var(--border);
  }
  .trow {
    border-bottom: 1px solid var(--border);
    font-size: 13px;
  }
  .trow:last-child {
    border-bottom: none;
  }
  .reason {
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px 9px;
    font-size: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .small {
    font-size: 12px;
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
  }
  .modal {
    width: 440px;
    max-width: 90vw;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .opts {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 8px 0;
  }
  .opt {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 13px;
  }
  .opt input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 6px;
  }
</style>
