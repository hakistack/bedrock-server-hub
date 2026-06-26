<script lang="ts">
  import { onMount } from 'svelte';
  import { api, onBackupProgress, openInFolder } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { formatDate, backupReasonLabel } from '$lib/util/format';
  import { humanSize } from '$lib/util/format';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
  import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
  import Select from '$lib/components/shared/Select.svelte';
  import NumberField from '$lib/components/shared/NumberField.svelte';
  import Toggle from '$lib/components/shared/Toggle.svelte';
  import type { BackupProgress, BackupRecord, BackupSchedule } from '$lib/types/backup';

  let backups = $state<BackupRecord[]>([]);
  let loading = $state(false);
  let busy = $state(false);
  let loadedFor = $state<string | null>(null);

  let schedule = $state<BackupSchedule | null>(null);
  let intervalStr = $state('60');
  let retentionStr = $state('7');
  let savingSchedule = $state(false);

  let restoring = $state<BackupRecord | null>(null);
  let restoreWorld = $state(true);
  let restoreProperties = $state(true);

  let deleting = $state<BackupRecord | null>(null);
  let deletingBusy = $state(false);

  let progress = $state<BackupProgress | null>(null);

  const server = $derived(serverStore.selected);
  const pct = $derived(progress && progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : null);

  function reasonTone(reason: string): 'default' | 'info' | 'gold' | 'warning' {
    if (reason === 'scheduled') return 'info';
    if (reason === 'manual') return 'gold';
    if (reason === 'pre_restore') return 'warning';
    return 'default';
  }

  onMount(() => {
    let unlisten: (() => void) | null = null;
    onBackupProgress((p) => {
      if (p.phase === 'completed') progress = null;
      else if (p.phase === 'zipping') progress = p;
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
  async function reload() {
    if (serverStore.selectedId) backups = await api.listBackups(serverStore.selectedId);
  }

  async function createManual() {
    const id = serverStore.selectedId;
    if (!id || busy) return;
    busy = true;
    try {
      await api.createBackup(id);
      await reload();
      toasts.success('Backup manual creado.');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
      progress = null;
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
      toasts.success('Programación guardada.');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      savingSchedule = false;
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

  async function confirmDelete() {
    const b = deleting;
    if (!b) return;
    deletingBusy = true;
    try {
      await api.deleteBackup(b.id);
      await reload();
      toasts.success('Backup eliminado.');
      deleting = null;
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      deletingBusy = false;
    }
  }
</script>

<PageHeader title="Backups" subtitle="Copias de seguridad del servidor y sus mundos.">
  {#snippet actions()}
    {#if server}<Button variant="primary" onclick={createManual} loading={busy}>+ Backup manual</Button>{/if}
  {/snippet}
</PageHeader>

{#if progress}
  <Card padding="sm">
    <div class="row spread" style="margin-bottom:8px;">
      <span>Comprimiendo mundo…</span>
      <span class="mono faint">{progress.done}/{progress.total}{pct != null ? ` · ${pct}%` : ''}</span>
    </div>
    <ProgressBar value={pct} indeterminate={pct == null} />
  </Card>
{/if}

{#if !server}
  <div class="card"><EmptyState icon="💾" title="Sin servidor" description="Selecciona un servidor para ver sus backups." /></div>
{:else}
  {#if schedule}
    <Card title="Backups automáticos">
      {#snippet actions()}
        <Toggle checked={schedule!.enabled} onToggle={(v) => (schedule!.enabled = v)} />
      {/snippet}
      {#snippet children()}
        {#if schedule!.enabled}
          <div class="sched">
            <div class="field">
              <span class="lbl">Modo</span>
              <Select
                bind:value={schedule!.mode}
                options={[
                  { value: 'interval', label: 'Por intervalo' },
                  { value: 'daily', label: 'Diario (hora fija)' },
                ]}
                ariaLabel="Modo"
              />
            </div>
            {#if schedule!.mode === 'daily'}
              <div class="field"><span class="lbl">Hora (HH:MM)</span><input class="input mono" type="time" bind:value={schedule!.dailyTime} /></div>
            {:else}
              <div class="field"><span class="lbl">Cada (min)</span><NumberField bind:value={intervalStr} min={5} step={5} /></div>
            {/if}
            <div class="field"><span class="lbl">Retención</span><NumberField bind:value={retentionStr} min={0} /></div>
          </div>
          <p class="faint small">Backup del mundo activo; conserva los últimos N. Requiere la app abierta.</p>
        {/if}
        <div class="row" style="justify-content:flex-end; margin-top:10px;">
          <Button variant="primary" onclick={saveSchedule} loading={savingSchedule}>Guardar programación</Button>
        </div>
      {/snippet}
    </Card>
  {/if}

  {#if loading}
    <div class="card" style="margin-top:16px;"><Spinner text="Cargando backups…" /></div>
  {:else if backups.length === 0}
    <div class="card" style="margin-top:16px;">
      <EmptyState icon="💾" title="Sin backups todavía" description="Se crean automáticamente antes de operaciones que modifican mundos o configuración, o manualmente desde aquí." />
    </div>
  {:else}
    <div class="list" style="margin-top:16px;">
      {#each backups as b (b.id)}
        <div class="bk">
          <div class="bk-main">
            <Badge tone={reasonTone(b.reason)}>{backupReasonLabel(b.reason)}</Badge>
            <div class="bk-info">
              <strong>{formatDate(b.createdAt)}</strong>
              <div class="faint small">
                {b.worldName ?? 'solo configuración'}{b.sizeBytes != null ? ` · ${humanSize(b.sizeBytes)}` : ''}
              </div>
            </div>
          </div>
          <div class="bk-actions">
            <Button size="sm" onclick={() => openRestore(b)} disabled={busy}>↺ Restaurar</Button>
            <Button size="sm" onclick={() => openInFolder(b.path)}>📂</Button>
            <Button size="sm" variant="danger" onclick={() => (deleting = b)} disabled={busy}>Eliminar</Button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<!-- Restore options modal -->
{#if restoring}
  <div class="overlay" role="button" tabindex="0" onclick={() => !busy && (restoring = null)} onkeydown={(e) => e.key === 'Escape' && !busy && (restoring = null)}>
    <div class="modal" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <h2>Restaurar backup</h2>
      <p class="muted small">{backupReasonLabel(restoring.reason)} · {formatDate(restoring.createdAt)}</p>
      <p class="faint small">Se creará un backup de seguridad del estado actual antes de restaurar.</p>
      <div class="opts">
        <label class="opt"><input type="checkbox" bind:checked={restoreWorld} disabled={!restoring.worldName} /> Restaurar mundo {restoring.worldName ? `("${restoring.worldName}")` : '(no incluido)'}</label>
        <label class="opt"><input type="checkbox" bind:checked={restoreProperties} /> Restaurar server.properties</label>
      </div>
      <div class="row" style="justify-content:flex-end; gap:10px;">
        <Button onclick={() => (restoring = null)} disabled={busy}>Cancelar</Button>
        <Button variant="primary" loading={busy} onclick={confirmRestore}>Restaurar</Button>
      </div>
    </div>
  </div>
{/if}

<ConfirmDialog
  open={deleting !== null}
  title="Eliminar backup"
  message="¿Eliminar este backup de forma permanente?"
  confirmLabel="Eliminar"
  danger
  busy={deletingBusy}
  onconfirm={confirmDelete}
/>

<style>
  .sched {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
    margin-bottom: 6px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .lbl {
    font-size: 13px;
    font-weight: 550;
  }
  .small {
    font-size: 12px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .bk {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 13px 16px;
  }
  .bk-main {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .bk-info strong {
    font-weight: 550;
    font-size: 14px;
  }
  .bk-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 700;
    backdrop-filter: blur(2px);
  }
  .modal {
    width: 440px;
    max-width: 90vw;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .opts {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 6px 0;
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
  @media (max-width: 760px) {
    .sched {
      grid-template-columns: 1fr;
    }
  }
</style>
