<script lang="ts">
  import { page } from '$app/stores';
  import { afterNavigate } from '$app/navigation';

  import { activeLabel } from '$lib/config/nav';
  import UpdateBanner from '$lib/components/shared/UpdateBanner.svelte';

  let navIdx = $state(0);
  let navMax = $state(0);
  const canBack = $derived(navIdx > 0);
  const canForward = $derived(navIdx < navMax);
  const crumb = $derived(activeLabel($page.url.pathname));

  afterNavigate((nav) => {
    if (nav.type === 'enter') {
      navIdx = 0;
      navMax = 0;
    } else if (nav.type === 'popstate') {
      const delta = (nav as { delta?: number }).delta ?? 0;
      navIdx = Math.max(0, Math.min(navMax, navIdx + delta));
    } else {
      navIdx += 1;
      navMax = navIdx;
    }
  });
</script>

<div class="topbar">
  <div class="arrows">
    <button class="arrow" onclick={() => history.back()} disabled={!canBack} aria-label="Atrás" title="Atrás">
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <path d="M10 3l-5 5 5 5" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <button class="arrow" onclick={() => history.forward()} disabled={!canForward} aria-label="Adelante" title="Adelante">
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
  </div>
  {#if crumb}<span class="crumb">{crumb}</span>{/if}
  <div class="right"><UpdateBanner /></div>
</div>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 32px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    background: var(--bg);
  }
  .arrows {
    display: flex;
    gap: 6px;
  }
  .arrow {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    transition: background 0.12s, color 0.12s;
  }
  .arrow:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text);
  }
  .arrow:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .crumb {
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 600;
  }
  .right {
    margin-left: auto;
  }
</style>
