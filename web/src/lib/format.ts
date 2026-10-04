import type { ChannelEntry, NodeEntry } from './types'
import { parseServerTime } from './time.svelte'

export const nodeName = (n: Pick<NodeEntry, 'verbose_name' | 'node_id'>): string =>
  n.verbose_name?.trim() || n.node_id

export const channelName = (c: Pick<ChannelEntry, 'verbose_name' | 'channel_id'>): string =>
  c.verbose_name?.trim() || `Channel ${c.channel_id}`

export function fmtNum(v: number | null | undefined, decimals = 0): string {
  return v === null || v === undefined || !Number.isFinite(v) ? '–' : v.toFixed(decimals)
}

/** "just now", "12 s ago", "5 min ago", "3 h ago", "2 d ago" */
export function ago(thenMs: number, nowMs: number): string {
  const s = Math.max(0, Math.round((nowMs - thenMs) / 1000))
  if (s < 5) return 'just now'
  if (s < 60) return `${s} s ago`
  if (s < 3600) return `${Math.round(s / 60)} min ago`
  if (s < 86400) return `${Math.round(s / 3600)} h ago`
  return `${Math.round(s / 86400)} d ago`
}

/** 90 -> "90 s", 600 -> "10 min", 7200 -> "2 h" */
export function fmtDuration(secs: number): string {
  if (secs >= 3600 && secs % 3600 === 0) return `${secs / 3600} h`
  if (secs >= 60 && secs % 60 === 0) return `${secs / 60} min`
  return `${secs} s`
}

export const agoFromServer = (ts: string, nowMs: number): string => ago(parseServerTime(ts), nowMs)

export type NodeStatus = 'online' | 'offline'

/**
 * A node has no heartbeat yet: `last_active` only moves when telemetry arrives.
 * So "online" means a report arrived within three report intervals (plus slack).
 */
export function nodeStatus(node: NodeEntry, serverNowMs: number): NodeStatus {
  const ageS = (serverNowMs - parseServerTime(node.last_active)) / 1000
  return ageS <= node.telemetry_report_freq * 3 + 15 ? 'online' : 'offline'
}
