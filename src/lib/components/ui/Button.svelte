<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'default' | 'danger' | 'ghost';
  type Size = 'sm' | 'md' | 'lg';

  let {
    variant = 'default',
    size = 'md',
    type = 'button',
    href,
    disabled = false,
    loading = false,
    full = false,
    onclick,
    title,
    ariaLabel,
    children,
  }: {
    variant?: Variant;
    size?: Size;
    type?: 'button' | 'submit';
    href?: string;
    disabled?: boolean;
    loading?: boolean;
    full?: boolean;
    onclick?: (e: MouseEvent) => void;
    title?: string;
    ariaLabel?: string;
    children?: Snippet;
  } = $props();
</script>

{#if href}
  <a class="ui-btn v-{variant} s-{size}" class:full {href} {title} aria-label={ariaLabel}>
    {@render children?.()}
  </a>
{:else}
  <button
    class="ui-btn v-{variant} s-{size}"
    class:full
    {type}
    disabled={disabled || loading}
    {onclick}
    {title}
    aria-label={ariaLabel}
  >
    {#if loading}<span class="spinner" aria-hidden="true"></span>{/if}
    {@render children?.()}
  </button>
{/if}

<style>
  .ui-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    border-radius: var(--radius-sm);
    font-weight: 550;
    font-family: inherit;
    transition: background 0.15s, border-color 0.15s, opacity 0.15s, transform 0.06s;
    white-space: nowrap;
    text-decoration: none;
  }
  .ui-btn:hover:not(:disabled) {
    background: var(--surface-3);
    border-color: var(--border-strong);
  }
  .ui-btn:active:not(:disabled) {
    transform: translateY(1px);
  }
  .ui-btn:focus-visible {
    outline: none;
    box-shadow: var(--ring);
  }
  .ui-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .full {
    width: 100%;
  }

  .s-sm {
    padding: 5px 11px;
    font-size: 13px;
  }
  .s-md {
    padding: 9px 15px;
  }
  .s-lg {
    padding: 12px 20px;
    font-size: 15px;
  }

  .v-primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #04130d;
    font-weight: 650;
  }
  .v-primary:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
  .v-danger {
    background: transparent;
    border-color: var(--danger);
    color: #ff8a8a;
  }
  .v-danger:hover:not(:disabled) {
    background: var(--danger-soft);
  }
  .v-ghost {
    background: transparent;
    border-color: transparent;
    color: var(--text-muted);
  }
  .v-ghost:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text);
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
