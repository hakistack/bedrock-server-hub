<script lang="ts">
  let { text, size = 28 }: { text?: string; size?: number } = $props();
</script>

<div class="loading">
  <span class="spinner" style:width={`${size}px`} style:height={`${size}px`}></span>
  {#if text}<span class="muted">{text}</span>{/if}
</div>

<style>
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    color: var(--text-muted);
  }
  .spinner {
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
