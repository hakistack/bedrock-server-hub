<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { PROPERTY_FIELDS, FIELD_KEYS } from '$lib/config/propertyFields';
  import type { PropertyEntry, PropertyFieldSpec } from '$lib/types/properties';
  import Select from '$lib/components/shared/Select.svelte';
  import NumberField from '$lib/components/shared/NumberField.svelte';
  import Toggle from '$lib/components/shared/Toggle.svelte';

  function selectOptions(spec: PropertyFieldSpec): { value: string; label: string }[] {
    const opts = (spec.options ?? []).map((o) => ({ value: o, label: o }));
    // Keep an out-of-list current value visible.
    const current = values[spec.key];
    if (current && !(spec.options ?? []).includes(current)) {
      opts.push({ value: current, label: `${current} (actual)` });
    }
    return opts;
  }

  let loading = $state(false);
  let saving = $state(false);
  let loadError = $state<string | null>(null);

  // key -> current (edited) value, and the on-disk snapshot for diffing.
  let values = $state<Record<string, string>>({});
  let original = $state<Record<string, string>>({});
  let order = $state<string[]>([]);
  let loadedFor = $state<string | null>(null);
  let showAdvanced = $state(false);

  const fieldsPresent = $derived(PROPERTY_FIELDS.filter((f) => f.key in values));
  const advancedKeys = $derived(order.filter((k) => !FIELD_KEYS.has(k)));
  const dirtyKeys = $derived(order.filter((k) => values[k] !== original[k]));
  const dirty = $derived(dirtyKeys.length > 0);

  // Reload whenever the selected server changes.
  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) {
      load(id);
    }
  });

  async function load(id: string) {
    loading = true;
    loadError = null;
    try {
      const entries: PropertyEntry[] = await api.readProperties(id);
      const map: Record<string, string> = {};
      for (const e of entries) map[e.key] = e.value;
      values = { ...map };
      original = { ...map };
      order = entries.map((e) => e.key);
      loadedFor = id;
    } catch (err) {
      loadError = errorMessage(err);
    } finally {
      loading = false;
    }
  }

  async function save() {
    const id = serverStore.selectedId;
    if (!id || !dirty || saving) return;
    saving = true;
    try {
      const updates = dirtyKeys.map((key) => ({ key, value: values[key] }));
      const entries = await api.updateProperties(id, updates);
      const map: Record<string, string> = {};
      for (const e of entries) map[e.key] = e.value;
      original = { ...map };
      values = { ...map };
      toasts.success('server.properties guardado.');
    } catch (err) {
      toasts.error(errorMessage(err));
    } finally {
      saving = false;
    }
  }

  function reset() {
    values = { ...original };
  }

  function asBool(v: string): boolean {
    return v === 'true';
  }
</script>

<header class="page-head">
  <div>
    <h1>Settings</h1>
    <p class="muted">Editor visual de <span class="mono">server.properties</span>.</p>
  </div>
  {#if serverStore.selected}
    <div class="actions">
      <button class="btn" onclick={reset} disabled={!dirty || saving}>Descartar</button>
      <button class="btn btn-primary" onclick={save} disabled={!dirty || saving}>
        {saving ? 'Guardando…' : dirty ? `Guardar (${dirtyKeys.length})` : 'Guardado'}
      </button>
    </div>
  {/if}
</header>

{#if !serverStore.selected}
  <div class="card empty-state">Selecciona o importa un servidor para editar su configuración.</div>
{:else if loading}
  <div class="card muted">Cargando propiedades…</div>
{:else if loadError}
  <div class="card" style="border-color: var(--danger);">
    <strong style="color: var(--danger);">No se pudo leer server.properties</strong>
    <p class="muted">{loadError}</p>
  </div>
{:else}
  <section class="card">
    <div class="card-title">Propiedades principales</div>
    <div class="fields">
      {#each fieldsPresent as f (f.key)}
        {@const spec = f as PropertyFieldSpec}
        <div class="field">
          <span class="field-label">{spec.label}</span>
          {#if spec.control === 'boolean'}
            <Toggle
              checked={asBool(values[spec.key])}
              onToggle={(v) => (values[spec.key] = v ? 'true' : 'false')}
            />
          {:else if spec.control === 'select'}
            <Select
              bind:value={values[spec.key]}
              options={selectOptions(spec)}
              ariaLabel={spec.label}
            />
          {:else if spec.control === 'number'}
            <NumberField bind:value={values[spec.key]} ariaLabel={spec.label} />
          {:else}
            <input class="input" type="text" bind:value={values[spec.key]} aria-label={spec.label} />
          {/if}
          <span class="key mono faint">{spec.key}</span>
        </div>
      {/each}
    </div>
  </section>

  {#if advancedKeys.length}
    <section class="card advanced">
      <button class="advanced-toggle" onclick={() => (showAdvanced = !showAdvanced)}>
        <span class="card-title" style="margin:0;">
          Propiedades avanzadas ({advancedKeys.length})
        </span>
        <span>{showAdvanced ? '▲' : '▼'}</span>
      </button>
      {#if showAdvanced}
        <div class="fields raw">
          {#each advancedKeys as key (key)}
            <div class="field">
              <label for={`raw-${key}`} class="mono raw-key">{key}</label>
              <input id={`raw-${key}`} class="input mono" type="text" bind:value={values[key]} />
            </div>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
{/if}

<style>
  .page-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 22px;
    gap: 16px;
  }
  .actions {
    display: flex;
    gap: 10px;
  }
  .fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 18px;
  }
  .fields.raw {
    grid-template-columns: 1fr;
    margin-top: 14px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field > label,
  .field-label {
    font-size: 13px;
    font-weight: 500;
  }
  .key {
    font-size: 11px;
  }
  .raw-key {
    font-size: 12px;
  }
  .advanced {
    margin-top: 18px;
  }
  .advanced-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 0;
  }
  @media (max-width: 760px) {
    .fields {
      grid-template-columns: 1fr;
    }
  }
</style>
