<script lang="ts">
  let {
    value = $bindable(),
    min,
    max,
    step = 1,
    ariaLabel = '',
  }: {
    value: string;
    min?: number;
    max?: number;
    step?: number;
    ariaLabel?: string;
  } = $props();

  function clamp(n: number): number {
    if (min != null && n < min) n = min;
    if (max != null && n > max) n = max;
    return n;
  }

  function bump(direction: number) {
    const current = parseInt(value || '0', 10);
    const base = Number.isNaN(current) ? 0 : current;
    value = String(clamp(base + direction * step));
  }
</script>

<div class="num">
  <button type="button" class="num-btn" onclick={() => bump(-1)} tabindex="-1" aria-label="Disminuir">
    −
  </button>
  <input
    class="num-input"
    type="number"
    bind:value
    {min}
    {max}
    {step}
    aria-label={ariaLabel}
  />
  <button type="button" class="num-btn" onclick={() => bump(1)} tabindex="-1" aria-label="Aumentar">
    +
  </button>
</div>

<style>
  .num {
    display: flex;
    align-items: stretch;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    transition: border-color 0.12s;
  }
  .num:focus-within {
    border-color: var(--accent);
  }
  .num-input {
    flex: 1;
    min-width: 0;
    background: transparent;
    border: none;
    color: var(--text);
    padding: 8px 11px;
    outline: none;
    text-align: left;
    font-family: inherit;
    font-size: inherit;
    -moz-appearance: textfield;
    appearance: textfield;
  }
  /* Hide native spinners (we provide our own). */
  .num-input::-webkit-outer-spin-button,
  .num-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .num-btn {
    width: 38px;
    background: var(--surface-2);
    border: none;
    border-left: 1px solid var(--border);
    color: var(--text-muted);
    font-size: 17px;
    line-height: 1;
    transition: background 0.12s, color 0.12s;
  }
  .num-btn:first-child {
    border-left: none;
    border-right: 1px solid var(--border);
  }
  .num-btn:hover {
    background: #2b313d;
    color: var(--text);
  }
</style>
