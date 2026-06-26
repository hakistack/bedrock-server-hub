<script lang="ts">
  import Button from './Button.svelte';

  let {
    open = $bindable(false),
    title,
    message,
    confirmLabel = 'Confirmar',
    cancelLabel = 'Cancelar',
    danger = false,
    busy = false,
    onconfirm,
  }: {
    open?: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    danger?: boolean;
    busy?: boolean;
    onconfirm: () => void;
  } = $props();

  function close() {
    if (!busy) open = false;
  }
</script>

{#if open}
  <div
    class="overlay"
    role="button"
    tabindex="0"
    onclick={close}
    onkeydown={(e) => e.key === 'Escape' && close()}
  >
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={() => {}}
    >
      <div class="icon" class:danger>{danger ? '⚠' : '?'}</div>
      <h2>{title}</h2>
      <p class="msg">{message}</p>
      <div class="actions">
        <Button onclick={close} disabled={busy}>{cancelLabel}</Button>
        <Button variant={danger ? 'danger' : 'primary'} loading={busy} onclick={onconfirm}>
          {confirmLabel}
        </Button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 700;
    backdrop-filter: blur(2px);
  }
  .dialog {
    width: 420px;
    max-width: 90vw;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    padding: 26px;
    text-align: center;
  }
  .icon {
    width: 46px;
    height: 46px;
    margin: 0 auto 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-size: 22px;
    font-weight: 700;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .icon.danger {
    background: var(--danger-soft);
    color: #ff8a8a;
  }
  h2 {
    font-size: 18px;
  }
  .msg {
    margin: 10px 0 20px;
    color: var(--text-muted);
    white-space: pre-line;
  }
  .actions {
    display: flex;
    justify-content: center;
    gap: 10px;
  }
</style>
