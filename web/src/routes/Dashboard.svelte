<script lang="ts">
  import { nodeName, nodeStatus } from '../lib/format'
  import NodeOverview from '../lib/components/NodeOverview.svelte'
  import { store } from '../lib/store.svelte'
  import { serverNow } from '../lib/time.svelte'

  const sorted = $derived([...store.nodes].sort((a, b) => nodeName(a).localeCompare(nodeName(b))))
  const online = $derived(store.nodes.filter((n) => nodeStatus(n, serverNow()) === 'online').length)
  const enabledChannels = $derived(store.channels.filter((c) => c.enabled).length)

  $effect(() => {
    document.title = 'Flowers'
  })
</script>

<div class="head">
  <h1>Your plants</h1>
  {#if store.nodes.length}
    <p class="muted">
      {online} of {store.nodes.length} {store.nodes.length === 1 ? 'node is' : 'nodes are'} online,
      {enabledChannels} of {store.channels.length} {store.channels.length === 1 ? 'channel is' : 'channels are'} enabled.
    </p>
  {/if}
</div>

{#if !store.loaded}
  <p class="muted state">Loading nodes…</p>
{:else if !store.nodes.length}
  <section class="state empty">
    {#if store.unreachable}
      <h2>Can't reach the server</h2>
      <p class="muted">Check that the backend is running on port 3000. This page keeps trying every few seconds.</p>
    {:else}
      <h2>No nodes yet</h2>
      <p class="muted">
        Power on a node on the same Wi-Fi as the server. It registers itself when it boots, and shows up here within a few seconds.
      </p>
    {/if}
  </section>
{:else}
  {#each sorted as node (node.id)}
    <NodeOverview {node} />
  {/each}
{/if}

<style>
  .head {
    display: grid;
    gap: 6px;
    padding: 36px 0 28px;
  }
  .state {
    padding: 28px 0;
    border-top: 1px solid var(--rule);
  }
  .empty {
    display: grid;
    gap: 8px;
    max-width: 38rem;
  }
</style>
