<script lang="ts">
  import { toasts } from '$lib/stores/toast.store.svelte';
</script>

<div class="toast-wrap">
  {#each toasts.items as toast (toast.id)}
    <div class="toast {toast.kind}">
      <span>{toast.message}</span>
      <button class="close" onclick={() => toasts.dismiss(toast.id)} aria-label="Cerrar">×</button>
    </div>
  {/each}
</div>

<style>
  .toast-wrap {
    position: fixed;
    bottom: 18px;
    right: 18px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    z-index: 1000;
    max-width: 380px;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-left: 3px solid var(--info);
    box-shadow: var(--shadow);
    animation: slide-in 0.18s ease;
  }
  .toast.success {
    border-left-color: var(--accent);
  }
  .toast.error {
    border-left-color: var(--danger);
  }
  .close {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 18px;
    line-height: 1;
  }
  @keyframes slide-in {
    from {
      transform: translateX(20px);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
</style>
