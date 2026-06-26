<script lang="ts">
  export interface SelectOption {
    value: string;
    label: string;
  }

  let {
    value = $bindable(),
    options,
    placeholder = 'Selecciona…',
    ariaLabel = '',
    onChange,
  }: {
    value: string | null;
    options: SelectOption[];
    placeholder?: string;
    ariaLabel?: string;
    onChange?: (value: string) => void;
  } = $props();

  let open = $state(false);
  let highlight = $state(-1);
  let root = $state<HTMLDivElement | null>(null);

  const selectedLabel = $derived(options.find((o) => o.value === value)?.label ?? placeholder);
  const hasValue = $derived(options.some((o) => o.value === value));

  function openList() {
    open = true;
    highlight = Math.max(0, options.findIndex((o) => o.value === value));
  }
  function close() {
    open = false;
    highlight = -1;
  }
  function choose(o: SelectOption) {
    value = o.value;
    onChange?.(o.value);
    close();
  }

  function onKey(e: KeyboardEvent) {
    if (!open) {
      if (e.key === 'ArrowDown' || e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        openList();
      }
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlight = Math.min(options.length - 1, highlight + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlight = Math.max(0, highlight - 1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (options[highlight]) choose(options[highlight]);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      close();
    }
  }

  $effect(() => {
    if (!open) return;
    const onDoc = (e: MouseEvent) => {
      if (root && !root.contains(e.target as Node)) close();
    };
    document.addEventListener('mousedown', onDoc);
    return () => document.removeEventListener('mousedown', onDoc);
  });
</script>

<div class="select-root" bind:this={root}>
  <button
    type="button"
    class="select-trigger"
    class:open
    onclick={() => (open ? close() : openList())}
    onkeydown={onKey}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={ariaLabel}
  >
    <span class="value" class:placeholder={!hasValue}>{selectedLabel}</span>
    <svg class="chev" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
      <path
        d="M4 6l4 4 4-4"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  {#if open}
    <ul class="list" role="listbox">
      {#each options as o, i (o.value)}
        <li
          role="option"
          aria-selected={o.value === value}
          class="option"
          class:highlight={i === highlight}
          class:selected={o.value === value}
          onmouseenter={() => (highlight = i)}
          onmousedown={(e) => {
            e.preventDefault();
            choose(o);
          }}
        >
          <span>{o.label}</span>
          {#if o.value === value}<span class="check">✓</span>{/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .select-root {
    position: relative;
    width: 100%;
  }
  .select-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text);
    border-radius: var(--radius-sm);
    padding: 8px 11px;
    text-align: left;
    transition: border-color 0.12s, background 0.12s;
  }
  .select-trigger:hover {
    background: #1f242e;
  }
  .select-trigger.open,
  .select-trigger:focus-visible {
    border-color: var(--accent);
    outline: none;
  }
  .value {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .value.placeholder {
    color: var(--text-faint);
  }
  .chev {
    flex-shrink: 0;
    color: var(--text-muted);
    transition: transform 0.15s;
  }
  .select-trigger.open .chev {
    transform: rotate(180deg);
  }
  .list {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    margin: 0;
    padding: 5px;
    list-style: none;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow);
    z-index: 50;
    max-height: 260px;
    overflow-y: auto;
  }
  .option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }
  .option.highlight {
    background: #2b313d;
  }
  .option.selected {
    color: var(--accent);
  }
  .check {
    color: var(--accent);
    font-size: 12px;
  }
</style>
