<script lang="ts">
  import { waterChannel } from '../actions'
  import { channelKey, store } from '../store.svelte'
  import { clock } from '../time.svelte'
  import type { ChannelEntry, NodeEntry } from '../types'

  interface Props {
    node: NodeEntry
    channel: ChannelEntry
    primary?: boolean
  }
  let { node, channel, primary = true }: Props = $props()

  const left = $derived(
    Math.max(
      0,
      Math.ceil(((store.wateringUntil[channelKey(channel.node_id, channel.channel_id)] ?? 0) - clock.now) / 1000),
    ),
  )
  const cooling = $derived(left > 0)
</script>

<button
  class="btn water"
  class:primary
  type="button"
  disabled={!channel.enabled || cooling}
  title={channel.enabled ? undefined : 'Enable this channel to water it'}
  onclick={() => waterChannel(node, channel)}
>
  <svg width="14" height="16" viewBox="0 0 14 16" aria-hidden="true">
    <path d="M7 1C7 1 1.5 7 1.5 10.5a5.5 5.5 0 0 0 11 0C12.5 7 7 1 7 1Z" fill="currentColor" />
  </svg>
  {#if cooling}
    Sent ({left} s)
  {:else}
    Water now
  {/if}
</button>
