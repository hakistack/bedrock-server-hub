<script lang="ts">
  let {
    checked,
    onToggle,
    onLabel = 'Activado',
    offLabel = 'Desactivado',
  }: {
    checked: boolean;
    onToggle: (value: boolean) => void;
    onLabel?: string;
    offLabel?: string;
  } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  class="toggle"
  class:on={checked}
  onclick={() => onToggle(!checked)}
>
  <span class="track"><span class="knob"></span></span>
  <span class="label">{checked ? onLabel : offLabel}</span>
</button>

<style>
  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    background: none;
    border: none;
    padding: 4px 0;
    color: var(--text-muted);
    font-size: 13px;
  }
  .track {
    width: 40px;
    height: 22px;
    border-radius: 999px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    position: relative;
    transition: background 0.15s, border-color 0.15s;
    flex-shrink: 0;
  }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--text-faint);
    transition: transform 0.15s, background 0.15s;
  }
  .toggle.on .track {
    background: rgba(59, 165, 93, 0.25);
    border-color: var(--accent);
  }
  .toggle.on .knob {
    transform: translateX(18px);
    background: var(--accent);
  }
  .toggle.on .label {
    color: var(--accent);
  }
</style>
