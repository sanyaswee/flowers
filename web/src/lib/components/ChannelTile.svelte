<script lang="ts">
  import { ago, channelName, fmtNum } from '../format'
  import { nodeHref } from '../router.svelte'
  import { channelKey, store } from '../store.svelte'
  import { parseServerTime, serverNow } from '../time.svelte'
  import type { ChannelEntry, NodeEntry } from '../types'
  import SoilProbe from './SoilProbe.svelte'
  import WaterButton from './WaterButton.svelte'

  interface Props {
    node: NodeEntry
    channel: ChannelEntry
  }
  let { node, channel }: Props = $props()

  const latest = $derived(store.moistureLatest[channelKey(channel.node_id, channel.channel_id)])
  const value = $derived(latest?.soil_moisture ?? null)
  const measured = $derived(latest ? ago(parseServerTime(latest.timestamp), serverNow()) : null)
</script>

<div class="tile">
  <SoilProbe {value} height={116} dimmed={!channel.enabled} />
  <div class="body">
    <p class="value num" class:off={!channel.enabled}>{fmtNum(value)}{#if value !== null}<small>%</small>{/if}</p>
    <a class="name" href={nodeHref(node.node_id)}>{channelName(channel)}</a>
    <p class="meta small">
      {#if !channel.enabled}
        Off
      {:else if measured}
        Measured {measured}
      {:else}
        No reading yet
      {/if}
    </p>
    <WaterButton {node} {channel} primary={false} />
  </div>
</div>

<style>
  .tile {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }
  .body {
    display: grid;
    gap: 2px;
    justify-items: start;
    min-width: 0;
  }
  .value {
    font-size: 2rem;
    font-weight: 620;
    line-height: 1.05;
    letter-spacing: -0.02em;
  }
  .value.off {
    color: var(--ink-3);
  }
  small {
    margin-left: 2px;
    font-size: 1rem;
    font-weight: 500;
    color: var(--ink-2);
  }
  .name {
    font-weight: 600;
    color: var(--ink);
    text-decoration: none;
    overflow-wrap: anywhere;
  }
  .name:hover {
    text-decoration: underline;
  }
  .meta {
    color: var(--ink-2);
    margin-bottom: 8px;
  }
</style>
