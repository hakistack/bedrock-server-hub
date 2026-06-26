<script lang="ts">
  let {
    value,
    indeterminate = false,
    tone = 'accent',
  }: {
    value?: number | null;
    indeterminate?: boolean;
    tone?: 'accent' | 'gold';
  } = $props();

  const pct = $derived(Math.max(0, Math.min(100, value ?? 0)));
</script>

<div class="ui-progress" role="progressbar" aria-valuenow={indeterminate ? undefined : pct}>
  <div
    class="fill t-{tone}"
    class:indeterminate
    style:width={indeterminate ? '40%' : `${pct}%`}
  ></div>
</div>

<style>
  .ui-progress {
    width: 100%;
    height: 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 999px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
    border-radius: 999px;
    transition: width 0.2s ease;
  }
  .fill.t-gold {
    background: var(--gold);
  }
  .fill.indeterminate {
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
</style>
