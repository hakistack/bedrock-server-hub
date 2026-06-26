<script lang="ts">
  let {
    label,
    value,
    icon,
    tone = 'default',
  }: {
    label: string;
    value: string;
    icon?: string;
    tone?: 'default' | 'success' | 'warning' | 'danger';
  } = $props();
</script>

<div class="ui-stat t-{tone}">
  {#if icon}<div class="icon">{icon}</div>{/if}
  <div class="body">
    <div class="label">{label}</div>
    <div class="value">{value}</div>
  </div>
</div>

<style>
  .ui-stat {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 14px 16px;
  }
  .icon {
    width: 38px;
    height: 38px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 19px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-faint);
  }
  .value {
    font-size: 17px;
    font-weight: 650;
    margin-top: 2px;
  }
  .t-success .value {
    color: var(--accent);
  }
  .t-warning .value {
    color: var(--warning);
  }
  .t-danger .value {
    color: #ff8a8a;
  }
</style>
