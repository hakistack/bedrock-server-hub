<script lang="ts">
  import { api } from '$lib/api/commands';
  import { serverStore } from '$lib/stores/server.store.svelte';
  import { toasts } from '$lib/stores/toast.store.svelte';
  import { errorMessage } from '$lib/util/error';
  import { PROPERTY_FIELDS, PROPERTY_GROUPS, FIELD_KEYS } from '$lib/config/propertyFields';
  import type { PropertyEntry, PropertyFieldSpec } from '$lib/types/properties';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import SettingsField from '$lib/components/ui/SettingsField.svelte';
  import Select from '$lib/components/shared/Select.svelte';
  import NumberField from '$lib/components/shared/NumberField.svelte';
  import Toggle from '$lib/components/shared/Toggle.svelte';

  let loading = $state(false);
  let saving = $state(false);
  let loadError = $state<string | null>(null);
  let values = $state<Record<string, string>>({});
  let original = $state<Record<string, string>>({});
  let order = $state<string[]>([]);
  let loadedFor = $state<string | null>(null);
  let showAdvanced = $state(false);

  const advancedKeys = $derived(order.filter((k) => !FIELD_KEYS.has(k)));
  const dirtyKeys = $derived(order.filter((k) => values[k] !== original[k]));
  const dirty = $derived(dirtyKeys.length > 0);

  function fieldsInGroup(group: string): PropertyFieldSpec[] {
    return PROPERTY_FIELDS.filter((f) => f.group === group && f.key in values);
  }

  $effect(() => {
    const id = serverStore.selectedId;
    if (id && id !== loadedFor) load(id);
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

  function selectOptions(spec: PropertyFieldSpec) {
    const opts = (spec.options ?? []).map((o) => ({ value: o, label: o }));
    const current = values[spec.key];
    if (current && !(spec.options ?? []).includes(current)) {
      opts.push({ value: current, label: `${current} (actual)` });
    }
    return opts;
  }

  const asBool = (v: string) => v === 'true';
</script>

<PageHeader title="Settings" subtitle="Editor visual de server.properties.">
  {#snippet actions()}
    {#if serverStore.selected}
      <Button onclick={reset} disabled={!dirty || saving}>Descartar</Button>
      <Button variant="primary" onclick={save} loading={saving} disabled={!dirty}>
        {dirty ? `Guardar (${dirtyKeys.length})` : 'Guardado'}
      </Button>
    {/if}
  {/snippet}
</PageHeader>

{#if !serverStore.selected}
  <Card><EmptyState icon="⚙️" title="Sin servidor" description="Selecciona un servidor para editar su configuración." /></Card>
{:else if loading}
  <Card><Spinner text="Cargando propiedades…" /></Card>
{:else if loadError}
  <Card><p class="warn">No se pudo leer server.properties: {loadError}</p></Card>
{:else}
  <div class="sections">
    {#each PROPERTY_GROUPS as group (group)}
      {@const fields = fieldsInGroup(group)}
      {#if fields.length}
        <Card title={group}>
          <div class="fields">
            {#each fields as f (f.key)}
              <SettingsField label={f.label} hint={f.key} description={f.help}>
                {#if f.control === 'boolean'}
                  <Toggle
                    checked={asBool(values[f.key])}
                    onToggle={(v) => (values[f.key] = v ? 'true' : 'false')}
                  />
                {:else if f.control === 'select'}
                  <Select bind:value={values[f.key]} options={selectOptions(f)} ariaLabel={f.label} />
                {:else if f.control === 'number'}
                  <NumberField bind:value={values[f.key]} ariaLabel={f.label} />
                {:else}
                  <input class="input" type="text" bind:value={values[f.key]} aria-label={f.label} />
                {/if}
              </SettingsField>
            {/each}
          </div>
        </Card>
      {/if}
    {/each}

    {#if advancedKeys.length}
      <Card>
        <button class="adv-toggle" onclick={() => (showAdvanced = !showAdvanced)}>
          <span class="card-title" style="margin:0;">Otras propiedades ({advancedKeys.length})</span>
          <span>{showAdvanced ? '▲' : '▼'}</span>
        </button>
        {#if showAdvanced}
          <div class="raw">
            {#each advancedKeys as key (key)}
              <SettingsField label={key} hint="">
                <input class="input mono" type="text" bind:value={values[key]} aria-label={key} />
              </SettingsField>
            {/each}
          </div>
        {/if}
      </Card>
    {/if}
  </div>
{/if}

<style>
  .sections {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 20px;
  }
  .raw {
    display: grid;
    grid-template-columns: 1fr;
    gap: 14px;
    margin-top: 12px;
  }
  .adv-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 0;
  }
  .warn {
    color: var(--warning);
  }
  @media (max-width: 760px) {
    .fields {
      grid-template-columns: 1fr;
    }
  }
</style>
