<script lang="ts">
  import { onMount } from 'svelte'
  import Toasts from './lib/components/Toasts.svelte'
  import { ago } from './lib/format'
  import { homeHref, router } from './lib/router.svelte'
  import { store } from './lib/store.svelte'
  import { clock } from './lib/time.svelte'
  import Dashboard from './routes/Dashboard.svelte'
  import NodePage from './routes/NodePage.svelte'

  onMount(() => store.start())

  const updated = $derived(store.lastSync === null ? null : ago(store.lastSync, clock.now))
</script>

<header class="bar">
  <div class="page inner">
    <a class="brand" href={homeHref} aria-label="Flowers, all nodes">
      <svg width="26" height="26" viewBox="0 0 26 26" aria-hidden="true">
        <path d="M13 24V12" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" fill="none" />
        <path d="M13 13C13 7 9 4 3 4c0 6 3 9 10 9Z" fill="var(--c-tank)" />
        <path d="M13 11c0-5 3.5-8 10-8 0 5-3 8-10 8Z" fill="var(--c-moisture)" />
      </svg>
      Flowers
    </a>

    <div class="sync" role="status">
      {#if store.unreachable}
        <span class="bad">Can't reach the server. Retrying…</span>
      {:else if updated}
        <span class="muted small">Updated {updated}</span>
      {/if}
      <button class="btn quiet" type="button" onclick={() => store.refreshCore().then(() => store.refreshLatest())}>
        Refresh
      </button>
    </div>
  </div>
</header>

<main class="page">
  {#if router.route.name === 'node'}
    {#key router.route.nodeId}
      <NodePage nodeId={router.route.nodeId} />
    {/key}
  {:else}
    <Dashboard />
  {/if}
</main>

<Toasts />

<style>
  .bar {
    border-bottom: 1px solid var(--rule);
    background: var(--bg);
  }
  .inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 60px;
  }
  .brand {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 1.375rem;
    font-weight: 700;
    letter-spacing: -0.03em;
    color: var(--ink);
    text-decoration: none;
  }
  .sync {
    display: flex;
    align-items: center;
    gap: 8px;
    text-align: right;
  }
  .bad {
    color: var(--bad);
    font-size: 0.875rem;
    font-weight: 560;
  }
  main {
    padding-bottom: 96px;
  }
</style>
