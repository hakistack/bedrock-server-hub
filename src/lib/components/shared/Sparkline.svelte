<script lang="ts">
  let {
    values,
    max,
    color = 'var(--accent)',
    height = 48,
  }: {
    values: number[];
    max?: number;
    color?: string;
    height?: number;
  } = $props();

  const peak = $derived(Math.max(max ?? 1, ...values, 1));
  const points = $derived(
    values.length === 0
      ? ''
      : values
          .map((v, i) => {
            const n = values.length;
            const x = n === 1 ? 100 : (i / (n - 1)) * 100;
            const y = 100 - (v / peak) * 100;
            return `${x.toFixed(2)},${y.toFixed(2)}`;
          })
          .join(' '),
  );
  const area = $derived(points ? `0,100 ${points} 100,100` : '');
</script>

<svg class="spark" viewBox="0 0 100 100" preserveAspectRatio="none" style:height={`${height}px`}>
  {#if values.length > 1}
    <polygon points={area} fill={color} opacity="0.12" />
    <polyline
      points={points}
      fill="none"
      stroke={color}
      stroke-width="2"
      vector-effect="non-scaling-stroke"
      stroke-linejoin="round"
    />
  {/if}
</svg>

<style>
  .spark {
    width: 100%;
    display: block;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
</style>
