<script lang="ts">
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { api, pickFiles } from '$lib/api/commands';
  import { fileDrop } from '$lib/actions/fileDrop.svelte';
  import Select from '$lib/components/shared/Select.svelte';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { formatDate } from '$lib/util/format';
  import type { World } from '$lib/types/world';
  import type { AddonInstallReport, AddonPack, InstalledAddon } from '$lib/types/addon';

  interface StagedAddon {
    sourcePath: string;
    displayName: string;
    packs: AddonPack[];
    selected: Record<string, boolean>;
  }

  let staged = $state<StagedAddon[]>([]);
  let previewing = $state(false);
  let installing = $state(false);
  let dragHover = $state(false);

  let worlds = $state<World[]>([]);
  let activeLevel = $state('');
  let selectedWorld = $state<string | null>(null);
  let installed = $state<InstalledAddon[]>([]);
  let report = $state<AddonInstallReport | null>(null);
  let loadedFor = $state<string | null>(null);

  const server = $derived(serverStore.selected);

  function isSupported(p: AddonPack): boolean {
    return p.packType === 'behavior' || p.packType === 'resource';
  }

  // World options: existing worlds + the active level-name (which may not exist
  // yet on a brand-new server — addons can be pre-seeded before world gen).
  const worldOptions = $derived.by(() => {
    const opts = worlds.map((w) => ({
      value: w.name,
      label: w.isActive ? `${w.name} (activo)` : w.name,
    }));
    if (activeLevel && !worlds.some((w) => w.name === activeLevel)) {
      opts.unshift({ value: activeLevel, label: `${activeLevel} (se generará al iniciar)` });
    }
    return opts;
  });

  const totalSelected = $derived(
    staged.reduce(
      (sum, s) => sum + s.packs.filter((p) => isSupported(p) && s.selected[p.uuid]).length,
      0,
    ),
  );
  const canInstall = $derived(
    !!server && !!selectedWorld && totalSelected > 0 && !installing && !previewing,
  );

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) load(id);
  });

  async function load(id: string) {
    loadedFor = id;
    try {
      worlds = await api.listWorlds(id);
      const props = await api.readProperties(id);
      activeLevel = props.find((p) => p.key === 'level-name')?.value ?? 'Bedrock level';
      installed = await api.listInstalledAddons(id);
      // Default target: the active world (existing or to-be-generated).
      selectedWorld = worlds.find((w) => w.isActive)?.name ?? activeLevel;
    } catch (err) {
      toasts.error(errorMessage(err));
    }
  }

  async function addViaPicker() {
    if (previewing || installing) return;
    const paths = await pickFiles(['mcaddon', 'mcpack', 'zip'], 'Addons Bedrock');
    await addPaths(paths);
  }

  async function addPaths(paths: string[]) {
    if (!paths.length) return;
    previewing = true;
    try {
      for (const path of paths) {
        if (staged.some((s) => s.sourcePath === path)) continue;
        try {
          const pkg = await api.previewAddon(path);
          const selected: Record<string, boolean> = {};
          for (const p of pkg.packs) if (isSupported(p)) selected[p.uuid] = true;
          staged.push({
            sourcePath: path,
            displayName: pkg.displayName,
            packs: pkg.packs,
            selected,
          });
        } catch (err) {
          toasts.error(`${path.split(/[/\\]/).pop()}: ${errorMessage(err)}`);
        }
      }
    } finally {
      previewing = false;
    }
  }

  function removeStaged(sourcePath: string) {
    staged = staged.filter((s) => s.sourcePath !== sourcePath);
  }

  async function install() {
    if (!canInstall || !server || !selectedWorld) return;
    installing = true;
    report = null;
    try {
      const items = staged
        .map((s) => ({
          sourcePath: s.sourcePath,
          selectedUuids: s.packs
            .filter((p) => isSupported(p) && s.selected[p.uuid])
            .map((p) => p.uuid),
        }))
        .filter((i) => i.selectedUuids.length > 0);

      report = await api.installAddons(server.id, selectedWorld, items);
      installed = await api.listInstalledAddons(server.id);
      const ok = report.results.filter((r) => r.status === 'installed' || r.status === 'updated').length;
      toasts.success(`${ok} pack(s) instalados en "${selectedWorld}".`);
      staged = [];
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
  <p class="muted">
    Instala uno o varios <span class="mono">.mcaddon</span> / <span class="mono">.mcpack</span> de una vez.
  </p>
</header>

{#if !server}
  <div class="card empty-state">Selecciona o importa un servidor para instalar addons.</div>
{:else}
  <div class="grid">
    <section class="col">
      <button
        class="card dropzone"
        class:busy={previewing}
        class:drag-hover={dragHover}
        onclick={addViaPicker}
        use:fileDrop={{
          extensions: ['mcaddon', 'mcpack', 'zip'],
          onDrop: (p) => addPaths([p]),
          onHover: (h) => (dragHover = h),
        }}
      >
        <div class="dz-icon">🧩</div>
        {#if previewing}
          <p>Analizando…</p>
        {:else if dragHover}
          <p><strong>Suelta los archivos aquí</strong></p>
        {:else}
          <p><strong>Arrastra o selecciona addons</strong></p>
          <p class="faint">Puedes añadir varios · .mcaddon · .mcpack · .zip</p>
        {/if}
      </button>

      {#if staged.length}
        {#each staged as s (s.sourcePath)}
          <div class="card">
            <div class="row spread">
              <div class="card-title" style="margin:0;">{s.displayName}</div>
              <button class="btn btn-sm" onclick={() => removeStaged(s.sourcePath)}>Quitar</button>
            </div>
            <div class="packs">
              {#each s.packs as p (p.uuid)}
                {@const supported = p.packType === 'behavior' || p.packType === 'resource'}
                <label class="pack" class:unsupported={!supported}>
                  <input
                    type="checkbox"
                    disabled={!supported}
                    checked={supported && s.selected[p.uuid]}
                    onchange={(e) => (s.selected[p.uuid] = (e.target as HTMLInputElement).checked)}
                  />
                  <div class="pack-body">
                    <div class="pack-head">
                      <span class="ptype {p.packType}">{packLabel[p.packType]}</span>
                      <strong>{p.name}</strong>
                      <span class="faint mono ver">v{p.version.join('.')}</span>
                    </div>
                    {#if p.description}<p class="muted small desc">{p.description}</p>{/if}
                  </div>
                </label>
              {/each}
            </div>
          </div>
        {/each}

        <div class="card">
          <div class="field">
            <span class="field-label">Mundo destino</span>
            <Select
              bind:value={selectedWorld}
              options={worldOptions}
              placeholder="Selecciona un mundo…"
              ariaLabel="Mundo destino"
            />
          </div>
          <p class="faint small note">
            Se creará un backup automático antes de instalar. Si el mundo aún no existe, se prepara
            para que los packs se apliquen al generarlo en el primer arranque.
          </p>
          <button class="btn btn-primary install-btn" onclick={install} disabled={!canInstall}>
            {installing
              ? 'Instalando…'
              : `Instalar ${totalSelected} pack(s) de ${staged.length} addon(s)`}
          </button>
        </div>
      {/if}

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
    transition: border-color 0.15s, background 0.15s;
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
  .warnings {
    margin-top: 12px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
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
