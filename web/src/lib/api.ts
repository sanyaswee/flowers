import { clock, toNaiveUtc } from './time.svelte'
import type {
  ChannelEntry,
  ChannelTelemetryEntry,
  NodeEntry,
  NodeSettings,
  NodeTelemetryEntry,
  PlantSettings,
} from './types'

/** Empty by default: same origin, via the Vite proxy or a reverse proxy. Set VITE_API_BASE to override. */
const BASE = ((import.meta.env.VITE_API_BASE as string | undefined) ?? '').replace(/\/$/, '')

export class ApiError extends Error {
  readonly status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

function messageFor(status: number): string {
  switch (status) {
    case 400:
    case 422:
      return 'The server rejected those values.'
    case 404:
      return 'Not found. The node or channel is not in the database.'
    case 500:
      return 'The server hit a database error.'
    default:
      return `The server answered with status ${status}.`
  }
}

async function request<T>(method: 'GET' | 'POST', path: string, body?: unknown): Promise<T> {
  let res: Response
  try {
    res = await fetch(BASE + path, {
      method,
      headers: body === undefined ? undefined : { 'content-type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: AbortSignal.timeout(10_000),
    })
  } catch {
    throw new ApiError(0, "Can't reach the server.")
  }

  // The Date header has 1 s resolution, so add half a second to centre the error.
  const date = res.headers.get('date')
  if (date) {
    const t = Date.parse(date)
    if (!Number.isNaN(t)) clock.offset = t + 500 - Date.now()
  }

  if (!res.ok) throw new ApiError(res.status, messageFor(res.status))
  if (res.headers.get('content-type')?.includes('json')) return (await res.json()) as T
  return undefined as T
}

/** The single-item endpoints return "" instead of null for an unnamed node/channel. Treat both as unnamed. */
function name(v: string | null): string | null {
  return v && v.trim() ? v : null
}
const cleanNode = (n: NodeEntry): NodeEntry => ({ ...n, verbose_name: name(n.verbose_name) })
const cleanChannel = (c: ChannelEntry): ChannelEntry => ({ ...c, verbose_name: name(c.verbose_name) })

export interface TelemetryQuery {
  /** epoch ms */
  start?: number
  /** epoch ms */
  end?: number
  /** Server default is 100 */
  limit?: number
}

function query(q: TelemetryQuery): string {
  const p = new URLSearchParams()
  if (q.start !== undefined) p.set('start', toNaiveUtc(q.start))
  if (q.end !== undefined) p.set('end', toNaiveUtc(q.end))
  if (q.limit !== undefined) p.set('limit', String(q.limit))
  const s = p.toString()
  return s ? `?${s}` : ''
}

const node = (id: string) => `/api/nodes/${encodeURIComponent(id)}`
const channel = (id: string, ch: number) => `${node(id)}/channels/${ch}`

export const api = {
  nodes: async () => (await request<NodeEntry[]>('GET', '/api/nodes')).map(cleanNode),
  node: async (id: string) => cleanNode(await request<NodeEntry>('GET', node(id))),
  channels: async () => (await request<ChannelEntry[]>('GET', '/api/channels')).map(cleanChannel),
  channel: async (id: string, ch: number) =>
    cleanChannel(await request<ChannelEntry>('GET', channel(id, ch))),

  nodeTelemetry: (id: string, q: TelemetryQuery = {}) =>
    request<NodeTelemetryEntry[]>('GET', `${node(id)}/telemetry${query(q)}`),
  channelTelemetry: (id: string, ch: number, q: TelemetryQuery = {}) =>
    request<ChannelTelemetryEntry[]>('GET', `${channel(id, ch)}/telemetry${query(q)}`),

  renameNode: (id: string, verbose_name: string) =>
    request<void>('POST', `${node(id)}/verbose`, { verbose_name }),
  renameChannel: (id: string, ch: number, verbose_name: string) =>
    request<void>('POST', `${channel(id, ch)}/verbose`, { verbose_name }),

  enableChannel: (id: string, ch: number) => request<void>('POST', `${channel(id, ch)}/enable`),
  disableChannel: (id: string, ch: number) => request<void>('POST', `${channel(id, ch)}/disable`),
  setChannelSettings: (id: string, ch: number, settings: PlantSettings) =>
    request<void>('POST', `${channel(id, ch)}/settings`, settings),
  setNodeSettings: (id: string, settings: NodeSettings) =>
    request<void>('POST', `${node(id)}/settings`, settings),

  /** 200 only means the command was published to MQTT. The node sends no acknowledgement. */
  water: (id: string, ch: number) => request<void>('POST', `${channel(id, ch)}/water`),
}
