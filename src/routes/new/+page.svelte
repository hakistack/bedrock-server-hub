<script lang="ts">
  import { goto } from '$app/navigation';
  import {
    api,
    pickFolder,
    onDownloadProgress,
    openInBrowser,
    OFFICIAL_DOWNLOAD_PAGE,
    MINECRAFT_EULA,
    MINECRAFT_PRIVACY,
  } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { humanSize } from '$lib/util/format';
  import type { BedrockServer } from '$lib/types/server';
  import type {
    DownloadProgress,
    ServerDownloadOption,
  } from '$lib/types/download';

  type Step = 'choose' | 'configure' | 'progress' | 'installing' | 'done';
  let step = $state<Step>('choose');
  let busy = $state(false);

  // --- Configure step state ---
  let serverName = $state('');
  let installDir = $state<string | null>(null);
  let options = $state<ServerDownloadOption[]>([]);
  let selectedId = $state<string | null>(null);
  let optionsLoading = $state(false);
  let optionsError = $state<string | null>(null);
  let manualUrl = $state('');
  let eulaAccepted = $state(false);

  // --- Progress state ---
  let progress = $state<DownloadProgress | null>(null);
  let downloadId = $state<string | null>(null);
  let unlistenProgress: (() => void) | null = null;

  // --- Done state ---
  let createdServer = $state<BedrockServer | null>(null);

  const selectedOption = $derived(options.find((o) => o.id === selectedId) ?? null);
  const canDownload = $derived(
    !!serverName.trim() && !!installDir && !!selectedOption && eulaAccepted && !busy,
  );

  // ---------- Import existing ----------
  async function importExisting() {
    if (busy) return;
    busy = true;
    try {
      const path = await pickFolder();
      if (!path) return;
      const v = await api.validateServerFolder(path);
      if (!v.isValid) {
        toasts.error('Carpeta no válida: ' + v.issues.join(' '));
        return;
      }
      const server = await api.importServer(path);
      registerAndGo(server, `Servidor "${server.name}" importado.`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      busy = false;
    }
  }

  // ---------- Download official ----------
  async function goConfigure() {
    step = 'configure';
    await loadOptions();
  }

  async function loadOptions() {
    optionsLoading = true;
    optionsError = null;
    try {
      options = await api.getOfficialDownloadOptions();
      selectedId = options[0]?.id ?? null;
    } catch (err) {
      optionsError = errorMessage(err);
    } finally {
      optionsLoading = false;
    }
  }

  async function useManualUrl() {
    if (!manualUrl.trim()) return;
    try {
      const opt = await api.resolveManualDownloadUrl(manualUrl.trim());
      // Replace the list with just this option and select it.
      options = [opt];
      selectedId = opt.id;
      optionsError = null;
      toasts.success('URL oficial aceptada.');
    } catch (err) {
      toasts.error(errorMessage(err));
    }
  }

  async function chooseInstallDir() {
    const dir = await pickFolder('Selecciona una carpeta vacía para instalar el servidor');
    if (dir) installDir = dir;
  }

  function registerAndGo(server: BedrockServer, message: string) {
    serverStore.upsert(server);
    serverStore.select(server.id);
    serverStore.setStatus(server.id, 'offline');
    toasts.success(message);
    goto('/');
  }

  async function startDownloadAndInstall() {
    if (!canDownload || !selectedOption || !installDir) return;
    busy = true;
    progress = null;
    const id = crypto.randomUUID();
    downloadId = id;

    unlistenProgress = await onDownloadProgress((p) => {
      if (p.downloadId === id) progress = p;
    });

    step = 'progress';
    try {
      const pkg = await api.downloadBedrockServer(selectedOption, id, eulaAccepted);
      step = 'installing';
      const server = await api.installDownloadedServer(pkg, installDir, serverName.trim());
      cleanupProgress();
      createdServer = server;
      serverStore.upsert(server);
      serverStore.select(server.id);
      serverStore.setStatus(server.id, 'offline');
      step = 'done';
    } catch (err) {
      cleanupProgress();
      const msg = errorMessage(err);
      if (msg.toLowerCase().includes('cancel')) {
        toasts.info('Descarga cancelada.');
      } else {
        toasts.error(msg);
      }
      step = 'configure';
    } finally {
      busy = false;
    }
  }

  function cleanupProgress() {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
  }

  async function cancelDownload() {
    if (downloadId) await api.cancelDownload(downloadId);
  }

  // ---------- Done actions ----------
  async function startNow() {
    if (!createdServer) return;
    try {
      await api.startServer(createdServer.id);
    } catch (err) {
      toasts.error(errorMessage(err));
    }
    goto('/console');
  }
</script>

<header class="page-head">
  <h1>Nuevo servidor</h1>
  <p class="muted">Crea un servidor importando uno existente o descargando el oficial.</p>
</header>

<!-- ============ STEP: CHOOSE ============ -->
{#if step === 'choose'}
  <div class="choices">
    <button class="choice card" onclick={importExisting} disabled={busy}>
      <div class="choice-icon">📂</div>
      <h2>Importar servidor existente</h2>
      <p class="muted">
        Ya tienes una carpeta de Bedrock Dedicated Server. Selecciónala y la registramos.
      </p>
    </button>

    <button class="choice card" onclick={goConfigure} disabled={busy}>
      <div class="choice-icon">⬇️</div>
      <h2>Descargar servidor oficial</h2>
      <p class="muted">
        Descarga los archivos oficiales del Bedrock Dedicated Server desde la web de Minecraft, sin
        salir de la app.
      </p>
    </button>
  </div>
{/if}

<!-- ============ STEP: CONFIGURE ============ -->
{#if step === 'configure'}
  <div class="card form">
    <div class="field">
      <label for="srv-name">Nombre del servidor</label>
      <input id="srv-name" class="input" bind:value={serverName} placeholder="Mi servidor" />
    </div>

    <div class="field">
      <label for="srv-dir">Carpeta de instalación</label>
      <div class="dir-row">
        <input
          id="srv-dir"
          class="input mono"
          readonly
          value={installDir ?? ''}
          placeholder="Elige una carpeta vacía…"
        />
        <button class="btn" onclick={chooseInstallDir}>Elegir…</button>
      </div>
      <span class="faint hint">Debe ser una carpeta vacía o nueva — nunca sobrescribimos archivos.</span>
    </div>

    <div class="field">
      <span class="lbl">Versión / plataforma (fuente oficial de Minecraft)</span>
      {#if optionsLoading}
        <p class="muted">Resolviendo descargas oficiales…</p>
      {:else if optionsError}
        <div class="fallback">
          <p class="warn">No se pudieron resolver los enlaces automáticamente.</p>
          <p class="muted small">{optionsError}</p>
          <div class="fallback-actions">
            <button class="btn btn-sm" onclick={loadOptions}>↻ Reintentar</button>
            <button class="btn btn-sm" onclick={() => openInBrowser(OFFICIAL_DOWNLOAD_PAGE)}>
              Abrir página oficial
            </button>
          </div>
          <div class="manual">
            <input
              class="input mono"
              bind:value={manualUrl}
              placeholder="Pega aquí la URL oficial .zip (bedrockdedicatedserver/…)"
            />
            <button class="btn btn-sm" onclick={useManualUrl}>Usar URL</button>
          </div>
        </div>
      {:else}
        <div class="options">
          {#each options as opt (opt.id)}
            <label class="option" class:selected={selectedId === opt.id}>
              <input type="radio" name="opt" value={opt.id} bind:group={selectedId} />
              <div>
                <div class="opt-label">{opt.label}</div>
                <div class="faint mono opt-plat">{opt.platform} · {opt.channel}</div>
              </div>
            </label>
          {/each}
        </div>
      {/if}
    </div>

    <div class="field eula">
      <p class="muted small">
        Los archivos provienen de la web oficial de Minecraft. Antes de descargar debes aceptar:
        <button class="link" onclick={() => openInBrowser(MINECRAFT_EULA)}>Minecraft EULA</button>
        ·
        <button class="link" onclick={() => openInBrowser(MINECRAFT_PRIVACY)}>
          Política de Privacidad
        </button>
      </p>
      <label class="chk">
        <input type="checkbox" bind:checked={eulaAccepted} />
        Acepto el Minecraft EULA y la Política de Privacidad.
      </label>
    </div>

    <div class="form-actions">
      <button class="btn" onclick={() => (step = 'choose')} disabled={busy}>← Atrás</button>
      <button class="btn btn-primary" onclick={startDownloadAndInstall} disabled={!canDownload}>
        Descargar y crear servidor
      </button>
    </div>
  </div>
{/if}

<!-- ============ STEP: PROGRESS ============ -->
{#if step === 'progress'}
  <div class="card center">
    <h2>Descargando servidor oficial…</h2>
    {#if progress}
      <div class="bar">
        <div
          class="bar-fill"
          style:width={`${progress.percentage ?? (progress.totalBytes ? 0 : 100)}%`}
          class:indeterminate={progress.percentage == null}
        ></div>
      </div>
      <p class="muted mono">
        {humanSize(progress.bytesDownloaded)}{progress.totalBytes
          ? ` / ${humanSize(progress.totalBytes)}`
          : ''}
        {progress.percentage != null ? ` · ${progress.percentage.toFixed(0)}%` : ''}
      </p>
    {:else}
      <p class="muted">Iniciando…</p>
    {/if}
    <button class="btn btn-danger" onclick={cancelDownload}>Cancelar</button>
  </div>
{/if}

<!-- ============ STEP: INSTALLING ============ -->
{#if step === 'installing'}
  <div class="card center">
    <div class="spinner"></div>
    <h2>Instalando…</h2>
    <p class="muted">Extrayendo, validando y registrando el servidor.</p>
  </div>
{/if}

<!-- ============ STEP: DONE ============ -->
{#if step === 'done' && createdServer}
  <div class="card center">
    <div class="choice-icon">✅</div>
    <h2>¡Servidor creado!</h2>
    <p class="muted">
      <strong>{createdServer.name}</strong>
      {#if createdServer.serverVersion}· v{createdServer.serverVersion}{/if}
    </p>
    <p class="faint mono small">{createdServer.path}</p>
    <div class="done-actions">
      <button class="btn btn-primary" onclick={startNow}>▶ Iniciar servidor</button>
      <button class="btn" onclick={() => goto('/')}>Ir al Dashboard</button>
      <button class="btn" onclick={() => goto('/settings')}>Editar configuración</button>
    </div>
  </div>
{/if}

<style>
  .page-head {
    margin-bottom: 24px;
  }
  .choices {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
  }
  .choice {
    text-align: left;
    cursor: pointer;
    transition: border-color 0.15s, transform 0.1s;
    color: var(--text);
  }
  .choice:hover:not(:disabled) {
    border-color: var(--accent);
    transform: translateY(-2px);
  }
  .choice-icon {
    font-size: 34px;
    margin-bottom: 10px;
  }
  .choice h2 {
    margin-bottom: 8px;
  }
  .form {
    max-width: 640px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .field > label,
  .lbl {
    font-size: 13px;
    font-weight: 500;
  }
  .dir-row {
    display: flex;
    gap: 8px;
  }
  .dir-row .input {
    flex: 1;
  }
  .hint {
    font-size: 12px;
  }
  .options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .option {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 11px 13px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    cursor: pointer;
  }
  .option.selected {
    border-color: var(--accent);
  }
  .option input {
    accent-color: var(--accent);
  }
  .opt-label {
    font-weight: 500;
  }
  .opt-plat {
    font-size: 12px;
    margin-top: 2px;
  }
  .eula {
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 13px;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 13px;
    margin-top: 8px;
  }
  .chk input {
    width: 17px;
    height: 17px;
    accent-color: var(--accent);
  }
  .link {
    background: none;
    border: none;
    color: var(--accent);
    text-decoration: underline;
    padding: 0;
    font-size: inherit;
  }
  .small {
    font-size: 12px;
  }
  .warn {
    color: var(--warning);
    margin: 0 0 4px;
  }
  .fallback {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px;
    background: var(--surface-2);
  }
  .fallback-actions {
    display: flex;
    gap: 8px;
    margin: 10px 0;
  }
  .manual {
    display: flex;
    gap: 8px;
  }
  .manual .input {
    flex: 1;
  }
  .form-actions {
    display: flex;
    justify-content: space-between;
    gap: 10px;
  }
  .center {
    max-width: 560px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    padding: 36px;
  }
  .bar {
    width: 100%;
    height: 12px;
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
  .spinner {
    width: 34px;
    height: 34px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .done-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    justify-content: center;
    margin-top: 8px;
  }
  @media (max-width: 800px) {
    .choices {
      grid-template-columns: 1fr;
    }
  }
</style>
