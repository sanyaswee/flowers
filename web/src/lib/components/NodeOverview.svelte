<script lang="ts">
  import { agoFromServer, nodeName, nodeStatus } from '../format'
  import { nodeHref } from '../router.svelte'
  import { store } from '../store.svelte'
  import { serverNow } from '../time.svelte'
  import type { NodeEntry } from '../types'
  import ChannelTile from './ChannelTile.svelte'
  import Readings from './Readings.svelte'
  import StatusPill from './StatusPill.svelte'

  interface Props {
    node: NodeEntry
  }
  let { node }: Props = $props()

  const status = $derived(nodeStatus(node, serverNow()))
  const channels = $derived(store.channelsOf(node.node_id))
  const hasReadings = $derived(
    node.water_tank || node.temperature || node.humidity || node.pressure || node.light,
  )
</script>

<section class="node" aria-labelledby="n-{node.id}">
  <header>
    <h2 id="n-{node.id}"><a href={nodeHref(node.node_id)}>{nodeName(node)}</a></h2>
    <StatusPill {status} detail="Last report {agoFromServer(node.last_active, serverNow())}" />
    <a class="btn quiet open" href={nodeHref(node.node_id)}>History and settings</a>
  </header>

  {#if hasReadings}
    <Readings {node} latest={store.nodeLatest[node.node_id]} dimmed={status === 'offline'} />
  {/if}

  {#if channels.length}
    <div class="tiles">
      {#each channels as channel (channel.id)}
        <ChannelTile {node} {channel} />
      {/each}
    </div>
  {:else}
    <p class="muted">No channels yet. They appear after the node's first settings sync.</p>
  {/if}
</section>

<style>
  .node {
    display: grid;
    gap: 22px;
    padding: 28px 0;
    border-top: 1px solid var(--rule);
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 6px 20px;
  }
  h2 a {
    color: var(--ink);
    text-decoration: none;
  }
  h2 a:hover {
    text-decoration: underline;
  }
  .open {
    margin-left: auto;
    align-self: center;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 230px), 1fr));
    gap: 28px 24px;
  }
</style>
