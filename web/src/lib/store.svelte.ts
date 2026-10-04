import { api } from './api'
import type { ChannelEntry, ChannelTelemetryEntry, NodeEntry, NodeTelemetryEntry } from './types'

const CORE_INTERVAL_MS = 5_000
const LATEST_INTERVAL_MS = 15_000

export const channelKey = (nodeId: string, channelId: number) => `${nodeId}/${channelId}`

/** App-wide data: node and channel lists, plus the newest reading for each. Polled while the tab is visible. */
class Store {
  nodes = $state<NodeEntry[]>([])
  channels = $state<ChannelEntry[]>([])
  /** Newest node telemetry row per node_id (null = none recorded yet) */
  nodeLatest = $state<Record<string, NodeTelemetryEntry | null>>({})
  /** Newest soil moisture row per "node_id/channel_id" */
  moistureLatest = $state<Record<string, ChannelTelemetryEntry | null>>({})
  /** epoch ms until which the water button of a channel is cooling down */
  wateringUntil = $state<Record<string, number>>({})

  loaded = $state(false)
  /** true when the last core refresh failed */
  unreachable = $state(false)
  /** browser epoch ms of the last successful refresh */
  lastSync = $state<number | null>(null)

  channelsOf(nodeId: string): ChannelEntry[] {
    return this.channels
      .filter((c) => c.node_id === nodeId)
      .sort((a, b) => a.channel_id - b.channel_id)
  }

  node(nodeId: string): NodeEntry | undefined {
    return this.nodes.find((n) => n.node_id === nodeId)
  }

  async refreshCore(): Promise<void> {
    try {
      const [nodes, channels] = await Promise.all([api.nodes(), api.channels()])
      this.nodes = nodes
      this.channels = channels
      this.unreachable = false
      this.lastSync = Date.now()
    } catch {
      this.unreachable = true
    } finally {
      this.loaded = true
    }
  }

  async refreshLatest(): Promise<void> {
    const nodes = this.nodes
    const channels = this.channels
    const [nodeRows, channelRows] = await Promise.all([
      Promise.all(
        nodes.map(async (n) => {
          try {
            const rows = await api.nodeTelemetry(n.node_id, { limit: 1 })
            return [n.node_id, rows[0] ?? null] as const
          } catch {
            return undefined
          }
        }),
      ),
      Promise.all(
        channels.map(async (c) => {
          try {
            const rows = await api.channelTelemetry(c.node_id, c.channel_id, { limit: 1 })
            return [channelKey(c.node_id, c.channel_id), rows[0] ?? null] as const
          } catch {
            return undefined
          }
        }),
      ),
    ])
    for (const r of nodeRows) if (r) this.nodeLatest[r[0]] = r[1]
    for (const r of channelRows) if (r) this.moistureLatest[r[0]] = r[1]
  }

  /** Start polling. Returns a stop function. */
  start(): () => void {
    let stopped = false

    const tickCore = async () => {
      if (stopped || document.hidden) return
      await this.refreshCore()
    }
    const tickLatest = async () => {
      if (stopped || document.hidden) return
      await this.refreshLatest()
    }
    const onVisible = async () => {
      if (document.hidden) return
      await this.refreshCore()
      await this.refreshLatest()
    }

    void (async () => {
      await this.refreshCore()
      await this.refreshLatest()
    })()

    const a = setInterval(tickCore, CORE_INTERVAL_MS)
    const b = setInterval(tickLatest, LATEST_INTERVAL_MS)
    document.addEventListener('visibilitychange', onVisible)

    return () => {
      stopped = true
      clearInterval(a)
      clearInterval(b)
      document.removeEventListener('visibilitychange', onVisible)
    }
  }
}

export const store = new Store()
