<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Stepper from '$lib/components/ui/Stepper.svelte';
  import FileDropzone from '$lib/components/ui/FileDropzone.svelte';
  import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
  import Select from '$lib/components/shared/Select.svelte';
  import type { AddonInstallReport, AddonPack, WorldPack, WorldPacks } from '$lib/types/addon';

  interface StagedAddon {
    sourcePath: string;
    displayName: string;
    packs: AddonPack[];
    selected: Record<string, boolean>;
  }

  const STEPS = ['Seleccionar', 'Mundo destino', 'Resultado'];
  let step = $state(0);

  let staged = $state<StagedAddon[]>([]);
  let previewing = $state(false);
  let installing = $state(false);

  let worlds = $state<{ name: string; isActive: boolean }[]>([]);
  let activeLevel = $state('');
  let selectedWorld = $state<string | null>(null);
  let worldPacks = $state<WorldPacks | null>(null);
  let report = $state<AddonInstallReport | null>(null);
  let loadedFor = $state<string | null>(null);
  let removing = $state<WorldPack | null>(null);
  let removingBusy = $state(false);

  const server = $derived(serverStore.selected);

  const isSupported = (p: AddonPack) => p.packType === 'behavior' || p.packType === 'resource';
  const totalSelected = $derived(
    staged.reduce((sum, s) => sum + s.packs.filter((p) => isSupported(p) && s.selected[p.uuid]).length, 0),
  );

  const worldOptions = $derived.by(() => {
    const opts = worlds.map((w) => ({ value: w.name, label: w.isActive ? `${w.name} (activo)` : w.name }));
    if (activeLevel && !worlds.some((w) => w.name === activeLevel)) {
      opts.unshift({ value: activeLevel, label: `${activeLevel} (se generará al iniciar)` });
    }
    return opts;
  });

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) load(id);
  });

  $effect(() => {
    const id = serverStore.selectedId;
    const w = selectedWorld;
    if (id && w) api.listWorldPacks(id, w).then((wp) => (worldPacks = wp)).catch(() => (worldPacks = null));
    else worldPacks = null;
  });

  async function load(id: string) {
    loadedFor = id;
    try {
      const w = await api.listWorlds(id);
      worlds = w.map((x) => ({ name: x.name, isActive: x.isActive }));
      const props = await api.readProperties(id);
      activeLevel = props.find((p) => p.key === 'level-name')?.value ?? 'Bedrock level';
      selectedWorld = w.find((x) => x.isActive)?.name ?? activeLevel;
    } catch (err) {
      toasts.error(errorMessage(err));
    }
  }
  async function reloadWorldPacks() {
    const id = serverStore.selectedId;
    if (id && selectedWorld) worldPacks = await api.listWorldPacks(id, selectedWorld).catch(() => null);
  }

  async function addPaths(paths: string[]) {
    if (!paths.length) return;
    previewing = true;
    try {
      for (const path of paths) {
        if (staged.some((s) => s.sourcePath === path)) continue;
        try {
          const pkg = await api.previewAddon(path);
          const sel: Record<string, boolean> = {};
          for (const p of pkg.packs) if (isSupported(p)) sel[p.uuid] = true;
          staged.push({ sourcePath: path, displayName: pkg.displayName, packs: pkg.packs, selected: sel });
        } catch (err) {
          toasts.error(`${path.split(/[/\\]/).pop()}: ${errorMessage(err)}`);
        }
      }
    } finally {
      previewing = false;
    }
  }
  function removeStaged(p: string) {
    staged = staged.filter((s) => s.sourcePath !== p);
  }

  async function install() {
    if (!server || !selectedWorld || totalSelected === 0 || installing) return;
    installing = true;
    report = null;
    step = 2;
    try {
      const items = staged
        .map((s) => ({
          sourcePath: s.sourcePath,
          selectedUuids: s.packs.filter((p) => isSupported(p) && s.selected[p.uuid]).map((p) => p.uuid),
        }))
        .filter((i) => i.selectedUuids.length > 0);
      report = await api.installAddons(server.id, selectedWorld, items);
      await reloadWorldPacks();
      staged = [];
    } catch (err) {
      toasts.error(errorMessage(err));
      step = 1;
    } finally {
      installing = false;
    }
  }

  function restart() {
    step = 0;
    report = null;
  }

  async function move(packType: 'behavior' | 'resource', index: number, dir: -1 | 1) {
    if (!server || !selectedWorld || !worldPacks) return;
    const list = packType === 'behavior' ? [...worldPacks.behavior] : [...worldPacks.resource];
    const t = index + dir;
    if (t < 0 || t >= list.length) return;
    [list[index], list[t]] = [list[t], list[index]];
    try {
      worldPacks = await api.reorderWorldPacks(server.id, selectedWorld, packType, list.map((p) => p.uuid));
    } catch (err) {
      toasts.error(errorMessage(err));
    }
  }
  async function confirmRemove() {
    if (!server || !selectedWorld || !removing) return;
    removingBusy = true;
    try {
      await api.uninstallAddon(server.id, selectedWorld, removing.uuid);
      await reloadWorldPacks();
      toasts.success(`"${removing.name}" eliminado.`);
      removing = null;
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      removingBusy = false;
    }
  }

  const packLabel: Record<string, string> = { behavior: 'Behavior', resource: 'Resource', skin: 'Skin', unknown: 'Desconocido' };
</script>

<PageHeader title="Addons" subtitle="Instala uno o varios .mcaddon / .mcpack en una corrida." />

{#if !server}
  <div class="card"><EmptyState icon="🧩" title="Sin servidor" description="Selecciona un servidor para instalar addons." /></div>
{:else}
  <div class="grid">
    <div class="col">
      <Card>
        <div class="stepper-wrap"><Stepper steps={STEPS} current={step} /></div>

        {#if step === 0}
          <FileDropzone
            extensions={['mcaddon', 'mcpack', 'zip']}
            name="Addons Bedrock"
            icon="🧩"
            label="Arrastra o selecciona addons"
            hint="Puedes añadir varios · .mcaddon · .mcpack · .zip"
            busy={previewing}
            onFile={(p) => addPaths([p])}
          />
          {#each staged as s (s.sourcePath)}
            <div class="staged">
              <div class="row spread">
                <strong>{s.displayName}</strong>
                <Button size="sm" onclick={() => removeStaged(s.sourcePath)}>Quitar</Button>
              </div>
              <div class="packs">
                {#each s.packs as p (p.uuid)}
                  {@const supported = isSupported(p)}
                  <label class="pack" class:unsupported={!supported}>
                    <input type="checkbox" disabled={!supported} checked={supported && s.selected[p.uuid]} onchange={(e) => (s.selected[p.uuid] = (e.target as HTMLInputElement).checked)} />
                    <div class="pk-body">
                      <div class="row" style="gap:8px;">
                        <Badge tone={p.packType === 'resource' ? 'success' : p.packType === 'behavior' ? 'info' : 'default'}>{packLabel[p.packType]}</Badge>
                        <strong>{p.name}</strong>
                        <span class="faint mono small ver">v{p.version.join('.')}</span>
                      </div>
                      {#if p.description}<p class="muted small desc">{p.description}</p>{/if}
                    </div>
                  </label>
                {/each}
              </div>
            </div>
          {/each}
          <div class="nav">
            <Button variant="primary" disabled={totalSelected === 0} onclick={() => (step = 1)}>
              Siguiente → ({totalSelected} pack{totalSelected === 1 ? '' : 's'})
            </Button>
          </div>
        {:else if step === 1}
          <div class="field">
            <span class="lbl">Mundo destino</span>
            <Select bind:value={selectedWorld} options={worldOptions} placeholder="Selecciona un mundo…" ariaLabel="Mundo destino" />
            <p class="faint small">Se crea un backup automático antes de instalar. Si el mundo no existe aún, se prepara para aplicar los packs al generarlo.</p>
          </div>
          <div class="nav spread">
            <Button onclick={() => (step = 0)}>← Atrás</Button>
            <Button variant="primary" loading={installing} disabled={!selectedWorld} onclick={install}>Instalar {totalSelected} pack(s)</Button>
          </div>
        {:else}
          {#if installing}
            <div class="installing"><div class="spinner"></div><p>Instalando…</p></div>
          {:else if report}
            <div class="result-head">✅ Instalación completada en "{report.worldName}"</div>
            {#each report.results as r (r.uuid)}
              <div class="result-row">
                <Badge tone={r.status === 'installed' ? 'success' : r.status === 'updated' ? 'info' : 'warning'}>{r.status}</Badge>
                <span>{r.name}</span>
              </div>
            {/each}
            {#if report.warnings.length}
              <div class="warnings">
                <strong class="warn small">⚠ Dependencias sin resolver</strong>
                {#each report.warnings as w (w)}<p class="muted small">{w}</p>{/each}
              </div>
            {/if}
            <div class="nav"><Button variant="primary" onclick={restart}>Instalar más</Button></div>
          {/if}
        {/if}
      </Card>
    </div>

    <div class="col">
      <Card title="Addons del mundo">
        {#snippet actions()}
          {#if selectedWorld}<span class="faint small mono">{selectedWorld}</span>{/if}
        {/snippet}
        {#snippet children()}
          {#snippet group(label: string, items: WorldPack[], packType: 'behavior' | 'resource')}
            {#if items.length}
              <div class="grp">
                <div class="grp-title">{label} ({items.length})</div>
                {#each items as p, i (p.uuid)}
                  <div class="wp">
                    <div class="ord">
                      <button class="o" onclick={() => move(packType, i, -1)} disabled={i === 0} aria-label="Subir">▲</button>
                      <button class="o" onclick={() => move(packType, i, 1)} disabled={i === items.length - 1} aria-label="Bajar">▼</button>
                    </div>
                    <div class="wp-info">
                      <strong>{p.name}</strong>
                      {#if !p.present}<Badge tone="warning">huérfano</Badge>{/if}
                      <div class="faint mono small">v{p.version.join('.')} · {p.uuid.slice(0, 8)}</div>
                    </div>
                    <Button size="sm" variant="danger" onclick={() => (removing = p)}>Quitar</Button>
                  </div>
                {/each}
              </div>
            {/if}
          {/snippet}
          {#if !selectedWorld}
            <p class="muted small">Selecciona un mundo.</p>
          {:else if worldPacks && (worldPacks.behavior.length || worldPacks.resource.length)}
            {@render group('Behavior packs', worldPacks.behavior, 'behavior')}
            {@render group('Resource packs', worldPacks.resource, 'resource')}
            <p class="faint small note">El orden define la prioridad: los de abajo se aplican por encima.</p>
          {:else}
            <p class="muted small">Este mundo no tiene addons todavía.</p>
          {/if}
        {/snippet}
      </Card>
    </div>
  </div>
{/if}

<ConfirmDialog
  open={removing !== null}
  title="Quitar addon"
  message={`¿Quitar "${removing?.name ?? ''}" del mundo "${selectedWorld ?? ''}"?\nSe creará un backup automático antes.`}
  confirmLabel="Quitar"
  danger
  busy={removingBusy}
  onconfirm={confirmRemove}
/>

<style>
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
  .stepper-wrap {
    margin-bottom: 20px;
  }
  .small {
    font-size: 12px;
  }
  .staged {
    margin-top: 14px;
    border-top: 1px solid var(--border);
    padding-top: 14px;
  }
  .packs {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 10px;
  }
  .pack {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 10px;
    background: var(--surface-2);
    cursor: pointer;
  }
  .pack input {
    margin-top: 3px;
    accent-color: var(--accent);
  }
  .pack.unsupported {
    opacity: 0.6;
    cursor: default;
  }
  .pk-body {
    flex: 1;
    min-width: 0;
  }
  .ver {
    margin-left: auto;
  }
  .desc {
    margin: 5px 0 0;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .lbl {
    font-size: 13px;
    font-weight: 550;
  }
  .nav {
    display: flex;
    margin-top: 18px;
    justify-content: flex-end;
  }
  .nav.spread {
    justify-content: space-between;
  }
  .installing {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 40px;
  }
  .spinner {
    width: 32px;
    height: 32px;
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
  .result-head {
    font-weight: 600;
    margin-bottom: 12px;
  }
  .result-row {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 7px 0;
    border-bottom: 1px solid var(--border);
    font-size: 13px;
  }
  .warnings {
    margin-top: 12px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }
  .warn {
    color: var(--warning);
  }
  .grp {
    margin-bottom: 14px;
  }
  .grp-title {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    margin-bottom: 6px;
  }
  .wp {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }
  .wp:last-child {
    border-bottom: none;
  }
  .ord {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .o {
    width: 22px;
    height: 16px;
    line-height: 1;
    font-size: 9px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-muted);
    border-radius: 4px;
  }
  .o:disabled {
    opacity: 0.35;
  }
  .wp-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .note {
    margin-top: 10px;
  }
  @media (max-width: 1000px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
