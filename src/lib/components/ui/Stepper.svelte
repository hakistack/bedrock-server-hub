<script lang="ts">
  let { steps, current }: { steps: string[]; current: number } = $props();
</script>

<ol class="stepper">
  {#each steps as label, i (label)}
    <li class="step" class:done={i < current} class:active={i === current}>
      <span class="dot">{i < current ? '✓' : i + 1}</span>
      <span class="label">{label}</span>
      {#if i < steps.length - 1}<span class="line"></span>{/if}
    </li>
  {/each}
</ol>

<style>
  .stepper {
    display: flex;
    list-style: none;
    margin: 0;
    padding: 0;
    gap: 0;
  }
  .step {
    display: flex;
    align-items: center;
    gap: 9px;
    flex: 1;
    color: var(--text-faint);
    font-size: 13px;
    min-width: 0;
  }
  .step:last-child {
    flex: 0;
  }
  .dot {
    width: 26px;
    height: 26px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--surface-2);
    font-size: 12px;
    font-weight: 600;
  }
  .label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .line {
    flex: 1;
    height: 1px;
    background: var(--border);
    margin: 0 6px;
  }
  .step.active {
    color: var(--text);
  }
  .step.active .dot {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-soft);
  }
  .step.done {
    color: var(--text-muted);
  }
  .step.done .dot {
    background: var(--accent);
    border-color: var(--accent);
    color: #04130d;
  }
  .step.done .line {
    background: var(--accent);
  }
</style>
