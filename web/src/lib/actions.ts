// Everything that changes something on the server. Each action reports the result with a
// toast, refreshes the lists, and returns whether it worked so forms can react.

import { api, ApiError } from './api'
import { channelName, nodeName, nodeStatus } from './format'
import { channelKey, store } from './store.svelte'
import { serverNowStatic } from './time.svelte'
import { toast } from './toast.svelte'
import {
  DEFAULT_PLANT_SETTINGS,
  MAX_PLANT_CHANNELS,
  type ChannelEntry,
  type NodeEntry,
  type NodeSettings,
  type PlantSettings,
} from './types'

function fail(title: string, e: unknown) {
  toast.error(title, e instanceof ApiError ? e.message : 'Something went wrong.')
}

async function run(title: string, success: string, fn: () => Promise<unknown>): Promise<boolean> {
  try {
    await fn()
  } catch (e) {
    fail(title, e)
    return false
  }
  toast.success(success)
  await store.refreshCore()
  return true
}

export const renameNode = (node: NodeEntry, name: string) =>
  run("Couldn't rename the node", 'Node renamed', () => api.renameNode(node.node_id, name.trim()))

export const renameChannel = (c: ChannelEntry, name: string) =>
  run("Couldn't rename the channel", 'Channel renamed', () =>
    api.renameChannel(c.node_id, c.channel_id, name.trim()),
  )

export const setChannelEnabled = (c: ChannelEntry, enabled: boolean) =>
  run(
    `Couldn't ${enabled ? 'enable' : 'disable'} ${channelName(c)}`,
    `${channelName(c)} ${enabled ? 'enabled' : 'disabled'}`,
    () => (enabled ? api.enableChannel(c.node_id, c.channel_id) : api.disableChannel(c.node_id, c.channel_id)),
  )

/** Saves timing for one channel. `enabled` is re-read first so a stale form can't flip it back. */
export const saveChannelSettings = (
  c: ChannelEntry,
  next: { moisture_m_freq_s: number; watering_time_s: number },
) =>
  run(`Couldn't save settings for ${channelName(c)}`, `Saved settings for ${channelName(c)}`, async () => {
    const fresh = await api.channel(c.node_id, c.channel_id)
    const settings: PlantSettings = { enabled: fresh.enabled, ...next }
    await api.setChannelSettings(c.node_id, c.channel_id, settings)
  })

/**
 * Saves node-level timing. The endpoint takes the whole settings struct and also overwrites every
 * channel's settings, so the current channel values are re-read and sent back unchanged.
 */
export const saveNodeSettings = (
  node: NodeEntry,
  next: { telemetry_packet_creation_freq_s: number; light_intensity_s: number; bmpe_s: number },
) =>
  run(`Couldn't save settings for ${nodeName(node)}`, `Saved settings for ${nodeName(node)}`, async () => {
    const channels = (await api.channels()).filter((c) => c.node_id === node.node_id)
    const plant_settings: PlantSettings[] = Array.from({ length: MAX_PLANT_CHANNELS }, (_, i) => {
      const c = channels.find((x) => x.channel_id === i)
      return c
        ? { enabled: c.enabled, moisture_m_freq_s: c.moisture_m_freq, watering_time_s: c.watering_time }
        : { ...DEFAULT_PLANT_SETTINGS }
    })
    const settings: NodeSettings = {
      telemetry_packet_creation_freq_s: next.telemetry_packet_creation_freq_s,
      plant_settings,
      m_freq: { light_intensity_s: next.light_intensity_s, bmpe_s: next.bmpe_s },
    }
    await api.setNodeSettings(node.node_id, settings)
  })

/**
 * Sends a water command. The server only publishes it over MQTT (no acknowledgement), so the
 * button cools down for the watering time plus a margin to stop double presses: the firmware
 * queues one extra run if a second command arrives while the pump is on.
 */
export async function waterChannel(node: NodeEntry, c: ChannelEntry): Promise<void> {
  const key = channelKey(c.node_id, c.channel_id)
  if ((store.wateringUntil[key] ?? 0) > Date.now()) return

  if (
    nodeStatus(node, serverNowStatic()) === 'offline' &&
    !confirm(`${nodeName(node)} hasn't reported recently. The command may be lost. Send it anyway?`)
  ) {
    return
  }

  try {
    await api.water(c.node_id, c.channel_id)
  } catch (e) {
    fail(`Couldn't water ${channelName(c)}`, e)
    return
  }
  store.wateringUntil[key] = Date.now() + (c.watering_time + 3) * 1000
  toast.success(`Water command sent to ${channelName(c)}`)
}
