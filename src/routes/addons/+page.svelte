<script lang="ts">
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { api, pickFile } from '$lib/api/commands';
  import { fileDrop } from '$lib/actions/fileDrop.svelte';
  import Select from '$lib/components/shared/Select.svelte';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { formatDate } from '$lib/util/format';
  import type { World } from '$lib/types/world';
  import type {
    AddonInstallReport,
    AddonPack,
    AddonPackage,
    InstalledAddon,
  } from '$lib/types/addon';

  let sourcePath = $state<string | null>(null);
  let preview = $state<AddonPackage | null>(null);
  let previewing = $state(false);
  let installing = $state(false);
  let dragHover = $state(false);

  let worlds = $state<World[]>([]);
  let selectedWorld = $state<string | null>(null);
  let installed = $state<InstalledAddon[]>([]);
  let report = $state<AddonInstallReport | null>(null);
  let loadedFor = $state<string | null>(null);

  // Per-pack install selection, keyed by uuid (only supported packs).
  let selected = $state<Record<string, boolean>>({});

  const server = $derived(serverStore.selected);

  function isSupported(p: AddonPack): boolean {
    return p.packType === 'behavior' || p.packType === 'resource';
  }

  const supportedPacks = $derived(preview ? preview.packs.filter(isSupported) : []);
  const selectedUuids = $derived(supportedPacks.filter((p) => selected[p.uuid]).map((p) => p.uuid));
  const canInstall = $derived(
    !!server && !!sourcePath && !!preview && !!selectedWorld && selectedUuids.length > 0 && !installing,
  );

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) load(id);
  });

  async function load(id: string) {
    loadedFor = id;
    try {
      worlds = await api.listWorlds(id);
      selectedWorld = worlds.find((w) => w.isActive)?.name ?? worlds[0]?.name ?? null;
      installed = await api.listInstalledAddons(id);
    } catch (err) {
      toasts.error(errorMessage(err));
    }
  }

  async function chooseFile() {
    if (previewing) return;
    const path = await pickFile(
      ['mcaddon', 'mcpack', 'zip'],
      'Addon Bedrock',
      'Selecciona un .mcaddon / .mcpack',
    );
    if (path) loadPreview(path);
  }

  async function loadPreview(path: string) {
    sourcePath = path;
    report = null;
    previewing = true;
    preview = null;
    selected = {};
    try {
      preview = await api.previewAddon(path);
      // Pre-select all supported packs.
      const sel: Record<string, boolean> = {};
      for (const p of preview.packs) {
        if (isSupported(p)) sel[p.uuid] = true;
      }
      selected = sel;
    } catch (err) {
      sourcePath = null;
      toasts.error(errorMessage(err));
    } finally {
      previewing = false;
    }
  }

  function clearSelection() {
    sourcePath = null;
    preview = null;
    report = null;
    selected = {};
  }

  async function install() {
    if (!canInstall || !server || !sourcePath || !selectedWorld) return;
    installing = true;
    report = null;
    try {
      report = await api.installAddon(server.id, selectedWorld, sourcePath, selectedUuids);
      installed = await api.listInstalledAddons(server.id);
      const ok = report.results.filter((r) => r.status === 'installed' || r.status === 'updated').length;
      toasts.success(`Addon instalado: ${ok} pack(s) en "${selectedWorld}".`);
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      installing = false;
    }
  }

  async function uninstall(a: InstalledAddon) {
    if (!server) return;
    const ok = await confirm(
      `¿Quitar "${a.name}" del mundo "${a.worldName}"?\nSe creará un backup automático antes.`,
      { title: 'Quitar addon', kind: 'warning' },
    );
    if (!ok) return;
    try {
      await api.uninstallAddon(server.id, a.worldName, a.uuid);
      installed = await api.listInstalledAddons(server.id);
      toasts.success(`"${a.name}" eliminado de "${a.worldName}".`);
    } catch (err) {
      toasts.error(errorMessage(err));
    }
  }

  const packLabel: Record<string, string> = {
    behavior: 'Behavior',
    resource: 'Resource',
    skin: 'Skin',
    unknown: 'Desconocido',
  };
</script>

<header class="page-head">
  <h1>Addons</h1>
  <p class="muted">Instala <span class="mono">.mcaddon</span> / <span class="mono">.mcpack</span> sin tocar JSON.</p>
</header>

{#if !server}
  <div class="card empty-state">Selecciona o importa un servidor para instalar addons.</div>
{:else}
  <div class="grid">
    <section class="col">
      <!-- Dropzone / selector -->
      <button
        class="card dropzone"
        class:busy={previewing}
        class:drag-hover={dragHover}
        onclick={chooseFile}
        use:fileDrop={{
          extensions: ['mcaddon', 'mcpack', 'zip'],
          onDrop: loadPreview,
          onHover: (h) => (dragHover = h),
        }}
      >
        <div class="dz-icon">🧩</div>
        {#if previewing}
          <p>Analizando addon…</p>
        {:else if dragHover}
          <p><strong>Suelta el archivo aquí</strong></p>
        {:else if sourcePath}
          <p class="mono small">{sourcePath}</p>
          <p class="faint">Click o arrastra otro archivo</p>
        {:else}
          <p><strong>Arrastra o selecciona un addon</strong></p>
          <p class="faint">.mcaddon · .mcpack · .zip</p>
        {/if}
      </button>

      <!-- Preview -->
      {#if preview}
        <div class="card">
          <div class="row spread">
            <div class="card-title" style="margin:0;">Contenido detectado · {preview.displayName}</div>
            <button class="btn btn-sm" onclick={clearSelection}>Limpiar</button>
          </div>
          <div class="packs">
            {#each preview.packs as p (p.uuid)}
              {@const supported = p.packType === 'behavior' || p.packType === 'resource'}
              <label class="pack" class:unsupported={!supported}>
                <input
                  type="checkbox"
                  disabled={!supported}
                  checked={supported && selected[p.uuid]}
                  onchange={(e) => (selected[p.uuid] = (e.target as HTMLInputElement).checked)}
                />
                <div class="pack-body">
                  <div class="pack-head">
                    <span class="ptype {p.packType}">{packLabel[p.packType]}</span>
                    <strong>{p.name}</strong>
                    <span class="faint mono ver">v{p.version.join('.')}</span>
                  </div>
                  {#if p.description}<p class="muted small desc">{p.description}</p>{/if}
                  <p class="faint mono uuid">{p.uuid}</p>
                </div>
              </label>
            {/each}
          </div>
          {#if supportedPacks.length === 0}
            <p class="warn small">Este addon no contiene packs behavior/resource instalables.</p>
          {/if}
        </div>

        <!-- Install controls -->
        <div class="card">
          <div class="field">
            <span class="field-label">Mundo destino</span>
            <Select
              bind:value={selectedWorld}
              options={worlds.map((w) => ({
                value: w.name,
                label: w.isActive ? `${w.name} (activo)` : w.name,
              }))}
              placeholder="Selecciona un mundo…"
              ariaLabel="Mundo destino"
            />
          </div>
          <p class="faint small note">
            Se creará un backup automático del mundo antes de instalar.
          </p>
          <button class="btn btn-primary install-btn" onclick={install} disabled={!canInstall}>
            {installing ? 'Instalando…' : `Instalar ${selectedUuids.length} pack(s)`}
          </button>
        </div>
      {/if}

      <!-- Result -->
      {#if report}
        <div class="card">
          <div class="card-title">Resultado</div>
          {#each report.results as r (r.uuid)}
            <div class="result-row">
              <span class="status {r.status}">{r.status}</span>
              <span>{r.name}</span>
              {#if r.message}<span class="faint small">— {r.message}</span>{/if}
            </div>
          {/each}
          {#if report.warnings.length}
            <div class="warnings">
              <strong class="warn small">⚠ Dependencias sin resolver</strong>
              {#each report.warnings as w (w)}
                <p class="muted small">{w}</p>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </section>

    <!-- Installed list -->
    <section class="col">
      <div class="card">
        <div class="card-title">Addons instalados</div>
        {#if installed.length === 0}
          <p class="muted small">Ninguno todavía.</p>
        {:else}
          {#each installed as a (a.id)}
            <div class="inst-row">
              <div>
                <strong>{a.name}</strong>
                <span class="ptype {a.packType}">{packLabel[a.packType] ?? a.packType}</span>
                <div class="faint mono small">v{a.version} · {a.worldName}</div>
              </div>
              <button class="btn btn-sm btn-danger" onclick={() => uninstall(a)}>Quitar</button>
            </div>
          {/each}
        {/if}
      </div>
    </section>
  </div>
{/if}

<style>
  .page-head {
    margin-bottom: 22px;
  }
  .grid {
    display: grid;
    grid-template-columns: 1.3fr 1fr;
    gap: 18px;
    align-items: start;
  }
  .col {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .dropzone {
    text-align: center;
    cursor: pointer;
    border-style: dashed;
    color: var(--text);
    transition: border-color 0.15s;
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    padding: 28px;
  }
  .dropzone:hover,
  .dropzone.drag-hover {
    border-color: var(--accent);
  }
  .dropzone.drag-hover {
    background: rgba(59, 165, 93, 0.08);
  }
  .dz-icon {
    font-size: 30px;
  }
  .small {
    font-size: 12px;
  }
  .packs {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 12px;
  }
  .pack {
    display: flex;
    align-items: flex-start;
    gap: 11px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 11px;
    background: var(--surface-2);
    cursor: pointer;
  }
  .pack input {
    margin-top: 3px;
    accent-color: var(--accent);
  }
  .pack-body {
    flex: 1;
    min-width: 0;
  }
  .pack.unsupported {
    opacity: 0.6;
    cursor: default;
  }
  .warnings {
    margin-top: 12px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }
  .pack-head {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .ver {
    margin-left: auto;
    font-size: 12px;
  }
  .desc {
    margin: 6px 0 0;
  }
  .uuid {
    margin: 6px 0 0;
    font-size: 11px;
    word-break: break-all;
  }
  .ptype {
    font-size: 11px;
    font-weight: 600;
    padding: 1px 8px;
    border-radius: 999px;
    border: 1px solid var(--border);
  }
  .ptype.behavior {
    color: #6fb1ff;
    border-color: #6fb1ff;
  }
  .ptype.resource {
    color: var(--accent);
    border-color: var(--accent);
  }
  .ptype.skin,
  .ptype.unknown {
    color: var(--text-muted);
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
    margin: 10px 0 12px;
  }
  .install-btn {
    width: 100%;
    justify-content: center;
  }
  .warn {
    color: var(--warning);
  }
  .result-row,
  .inst-row {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    font-size: 13px;
  }
  .inst-row {
    justify-content: space-between;
  }
  .result-row:last-child,
  .inst-row:last-child {
    border-bottom: none;
  }
  .status {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 8px;
    border-radius: 6px;
  }
  .status.installed {
    color: var(--accent);
    background: rgba(59, 165, 93, 0.12);
  }
  .status.updated {
    color: var(--info);
    background: rgba(79, 143, 240, 0.12);
  }
  .status.unsupported,
  .status.skipped {
    color: var(--warning);
    background: rgba(217, 164, 65, 0.12);
  }
  @media (max-width: 900px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
