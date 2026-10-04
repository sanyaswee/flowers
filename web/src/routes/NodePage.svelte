<script lang="ts">
  import { untrack } from 'svelte'
  import { renameNode } from '../lib/actions'
  import { api } from '../lib/api'
  import type { Pt } from '../lib/chartData'
  import Chart from '../lib/components/Chart.svelte'
  import ChannelPanel from '../lib/components/ChannelPanel.svelte'
  import DataTable from '../lib/components/DataTable.svelte'
  import EditableName from '../lib/components/EditableName.svelte'
  import NodeSettingsForm from '../lib/components/NodeSettingsForm.svelte'
  import RangeSelect, { RANGES, type RangeKey } from '../lib/components/RangeSelect.svelte'
  import Readings from '../lib/components/Readings.svelte'
  import StatusPill from '../lib/components/StatusPill.svelte'
  import { agoFromServer, nodeName, nodeStatus } from '../lib/format'
  import { homeHref } from '../lib/router.svelte'
  import { store } from '../lib/store.svelte'
  import { parseServerTime, serverNow, serverNowStatic } from '../lib/time.svelte'
  import type { ChannelTelemetryEntry, NodeEntry, NodeTelemetryEntry } from '../lib/types'

  interface Props {
    nodeId: string
  }
  let { nodeId }: Props = $props()

  /** Most rows requested per series. The API returns newest first, so a long range may only cover the latest part. */
  const LIMIT = 8000

  const node = $derived(store.node(nodeId))
  const channels = $derived(store.channelsOf(nodeId))
  const status = $derived(node ? nodeStatus(node, serverNow()) : 'offline')

  let range = $state<RangeKey>('6h')
  let loading = $state(false)
  let loadFailed = $state(false)
  let nodeRows = $state<NodeTelemetryEntry[]>([])
  let channelRows = $state<Record<number, ChannelTelemetryEntry[]>>({})
  let xDomain = $state<[number, number]>([Date.now() - RANGES['6h'].ms, Date.now()])
  let truncated = $state(false)
  let token = 0

  const toPts = <T extends { timestamp: string }>(rows: T[], pick: (r: T) => number | null): Pt[] => {
    const out: Pt[] = []
    for (let i = rows.length - 1; i >= 0; i--) {
      const v = pick(rows[i])
      if (v !== null && Number.isFinite(v)) out.push({ t: parseServerTime(rows[i].timestamp), v })
    }
    return out
  }

  async function load(n: NodeEntry, key: RangeKey) {
    const mine = ++token
    loading = true
    const end = serverNowStatic()
    const start = end - RANGES[key].ms
    try {
      const [rows, ...perChannel] = await Promise.all([
        api.nodeTelemetry(n.node_id, { start, end, limit: LIMIT }),
        ...Array.from({ length: n.n_channels }, (_, i) =>
          api.channelTelemetry(n.node_id, i, { start, end, limit: LIMIT }),
        ),
      ])
      if (mine !== token) return
      nodeRows = rows
      channelRows = Object.fromEntries(perChannel.map((r, i) => [i, r]))

      // If the row limit cut the range short, start the axis where the data starts.
      const cut = rows.length >= LIMIT
      truncated = cut
      const oldest = cut ? parseServerTime(rows[rows.length - 1].timestamp) : start
      xDomain = [Math.max(start, oldest), end]
      loadFailed = false
    } catch {
      if (mine === token) loadFailed = true
    } finally {
      if (mine === token) loading = false
    }
  }

  // True once the node record exists (a direct link can render before the first list arrives).
  const known = $derived(!!node)

  // Reload when the node or range changes, then every 15 s while the tab is visible.
  $effect(() => {
    if (!known) return
    const id = nodeId
    const key = range
    const first = untrack(() => store.node(id))
    // load() reads reactive state (the clock offset), which must not re-trigger this effect
    if (first) untrack(() => void load(first, key))
    const timer = setInterval(() => {
      const cur = untrack(() => store.node(id))
      if (cur && !document.hidden) untrack(() => void load(cur, key))
    }, 15_000)
    return () => clearInterval(timer)
  })

  $effect(() => {
    document.title = node ? `${nodeName(node)} – Flowers` : 'Flowers'
  })

  const temperature = $derived(toPts(nodeRows, (r) => r.temperature))
  const humidity = $derived(toPts(nodeRows, (r) => r.humidity))
  const pressure = $derived(toPts(nodeRows, (r) => r.pressure))
  const light = $derived(toPts(nodeRows, (r) => r.light_intensity))
  const tank = $derived(toPts(nodeRows, (r) => r.water_tank_level))

  const hasReadings = $derived(
    !!node && (node.water_tank || node.temperature || node.humidity || node.pressure || node.light),
  )
</script>

<p class="back"><a href={homeHref}>All nodes</a></p>

{#if !store.loaded}
  <p class="muted pad">Loading node…</p>
{:else if !node}
  <section class="pad missing">
    <h1>Node not found</h1>
    <p class="muted">There is no node called “{nodeId}”. It may not have booted since the server started.</p>
    <p><a class="btn" href={homeHref}>Back to all nodes</a></p>
  </section>
{:else}
  <header class="head">
    <EditableName
      level="h1"
      what="this node"
      display={nodeName(node)}
      value={node.verbose_name ?? ''}
      onsave={(name) => renameNode(node, name)}
    />
    <div class="meta">
      <StatusPill {status} detail="Last report {agoFromServer(node.last_active, serverNow())}" />
      <span class="muted small">ID {node.node_id}</span>
    </div>
  </header>

  <div class="toolbar">
    <span class="small muted">History</span>
    <RangeSelect bind:value={range} />
    {#if loadFailed}
      <span class="small err">Couldn't load history. Showing the last data that arrived.</span>
    {:else if truncated}
      <span class="small muted">Showing the latest {LIMIT.toLocaleString()} readings. Pick a shorter range for more detail.</span>
    {/if}
  </div>

  <section class="block" aria-labelledby="plants-h">
    <h2 id="plants-h">Plants</h2>
    {#if channels.length}
      {#each channels as channel (channel.id)}
        <ChannelPanel
          {node}
          {channel}
          points={toPts(channelRows[channel.channel_id] ?? [], (r) => r.soil_moisture)}
          {xDomain}
          {loading}
        />
      {/each}
    {:else}
      <p class="muted">No channels yet. They appear after the node's first settings sync.</p>
    {/if}
  </section>

  {#if hasReadings}
    <section class="block" aria-labelledby="air-h">
      <h2 id="air-h">Climate and water tank</h2>
      <Readings {node} latest={store.nodeLatest[node.node_id]} dimmed={status === 'offline'} />

      <div class="charts">
        {#if node.water_tank && node.water_tank_level}
          <div class="chart">
            <h3>Water tank level</h3>
            <Chart points={tank} {xDomain} {loading} label="Water tank level" unit="%" domain={[0, 100]} color="var(--c-tank)" />
            <DataTable label="Water tank level" unit="%" points={tank} />
          </div>
        {/if}
        {#if node.temperature}
          <div class="chart">
            <h3>Temperature</h3>
            <Chart points={temperature} {xDomain} {loading} label="Temperature" unit="°C" decimals={1} color="var(--c-temperature)" />
            <DataTable label="Temperature" unit="°C" decimals={1} points={temperature} />
          </div>
        {/if}
        {#if node.humidity}
          <div class="chart">
            <h3>Humidity</h3>
            <Chart points={humidity} {xDomain} {loading} label="Humidity" unit="%" domain={[0, 100]} color="var(--c-humidity)" />
            <DataTable label="Humidity" unit="%" points={humidity} />
          </div>
        {/if}
        {#if node.pressure}
          <div class="chart">
            <h3>Pressure</h3>
            <Chart points={pressure} {xDomain} {loading} label="Pressure" unit="hPa" decimals={0} color="var(--c-pressure)" />
            <DataTable label="Pressure" unit="hPa" points={pressure} />
          </div>
        {/if}
        {#if node.light}
          <div class="chart">
            <h3>Light</h3>
            <Chart points={light} {xDomain} {loading} label="Light" unit="lx" decimals={0} zeroBase color="var(--c-light)" />
            <DataTable label="Light" unit="lx" points={light} />
          </div>
        {/if}
      </div>
    </section>
  {/if}

  <section class="block" aria-labelledby="settings-h">
    <h2 id="settings-h">Node settings</h2>
    <NodeSettingsForm {node} />
  </section>

  <section class="block" aria-labelledby="about-h">
    <h2 id="about-h">About this node</h2>
    <dl class="about">
      <div><dt>ID</dt><dd>{node.node_id}</dd></div>
      <div><dt>Channels</dt><dd>{node.n_channels}</dd></div>
      <div><dt>Booted</dt><dd>{new Date(parseServerTime(node.last_boot)).toLocaleString()}</dd></div>
      <div><dt>Last report</dt><dd>{new Date(parseServerTime(node.last_active)).toLocaleString()}</dd></div>
    </dl>
  </section>
{/if}

<style>
  .back {
    padding: 20px 0 4px;
  }
  .pad {
    padding: 28px 0;
  }
  .missing {
    display: grid;
    gap: 12px;
  }
  .head {
    display: grid;
    gap: 8px;
    padding: 8px 0 24px;
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 20px;
  }
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 16px;
    padding: 12px 0 20px;
  }
  .err {
    color: var(--bad);
  }
  .block {
    display: grid;
    gap: 8px;
    padding: 28px 0 12px;
  }
  .block > h2 {
    margin-bottom: 4px;
  }
  .charts {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 420px), 1fr));
    gap: 28px 32px;
    margin-top: 20px;
  }
  .chart {
    min-width: 0;
  }
  .chart h3 {
    margin-bottom: 8px;
  }
  .about {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: 12px 24px;
    margin: 0;
  }
  .about dt {
    color: var(--ink-2);
    font-size: 0.875rem;
  }
  .about dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
</style>
