<script lang="ts">
  import { untrack } from 'svelte'
  import { renameChannel, saveChannelSettings, setChannelEnabled } from '../actions'
  import type { Pt } from '../chartData'
  import { ago, channelName, fmtNum } from '../format'
  import { channelKey, store } from '../store.svelte'
  import { parseServerTime, serverNow } from '../time.svelte'
  import type { ChannelEntry, NodeEntry } from '../types'
  import Chart from './Chart.svelte'
  import DataTable from './DataTable.svelte'
  import DurationField from './DurationField.svelte'
  import EditableName from './EditableName.svelte'
  import SoilProbe from './SoilProbe.svelte'
  import Switch from './Switch.svelte'
  import WaterButton from './WaterButton.svelte'

  interface Props {
    node: NodeEntry
    channel: ChannelEntry
    points: Pt[]
    xDomain: [number, number]
    loading: boolean
  }
  let { node, channel, points, xDomain, loading }: Props = $props()

  const latest = $derived(store.moistureLatest[channelKey(channel.node_id, channel.channel_id)])
  const value = $derived(latest?.soil_moisture ?? null)
  const measured = $derived(latest ? ago(parseServerTime(latest.timestamp), serverNow()) : null)

  // Timing form. The draft follows the server until the user edits it.
  // svelte-ignore state_referenced_locally
  let freq = $state(channel.moisture_m_freq)
  // svelte-ignore state_referenced_locally
  let wtime = $state(channel.watering_time)
  let freqOk = $state(true)
  let wtimeOk = $state(true)
  let saving = $state(false)
  let resetN = $state(0)

  const dirty = $derived(freq !== channel.moisture_m_freq || wtime !== channel.watering_time)

  $effect(() => {
    const f = channel.moisture_m_freq
    const w = channel.watering_time
    untrack(() => {
      if (!dirty) {
        freq = f
        wtime = w
      }
    })
  })

  async function save() {
    saving = true
    await saveChannelSettings(channel, { moisture_m_freq_s: freq, watering_time_s: wtime })
    saving = false
  }

  function reset() {
    freq = channel.moisture_m_freq
    wtime = channel.watering_time
    resetN++
  }
</script>

<article class="panel">
  <header>
    <EditableName
      level="h3"
      what={channelName(channel)}
      display={channelName(channel)}
      value={channel.verbose_name ?? ''}
      onsave={(name) => renameChannel(channel, name)}
    />
    <Switch
      checked={channel.enabled}
      label={channel.enabled ? 'Enabled' : 'Disabled'}
      onchange={(next) => setChannelEnabled(channel, next)}
    />
  </header>

  <div class="body">
    <div class="now">
      <SoilProbe {value} height={132} dimmed={!channel.enabled} />
      <div class="now-text">
        <p class="value num">{fmtNum(value)}{#if value !== null}<small>%</small>{/if}</p>
        <p class="small muted">
          {#if measured}Measured {measured}{:else}No reading yet{/if}
        </p>
        <WaterButton {node} {channel} />
      </div>
    </div>

    <div class="hist">
      <Chart
        {points}
        {xDomain}
        {loading}
        label="Soil moisture"
        unit="%"
        decimals={0}
        domain={[0, 100]}
        color="var(--c-moisture)"
      />
      <DataTable label="Soil moisture" unit="%" {points} />
    </div>
  </div>

  <details class="timing">
    <summary>Timing</summary>
    <form
      onsubmit={(e) => {
        e.preventDefault()
        void save()
      }}
    >
      <div class="fields">
        <DurationField
          bind:value={freq}
          bind:valid={freqOk}
          resetSignal={resetN}
          label="Measure soil moisture every"
          hint="How often the sensor is read."
        />
        <DurationField
          bind:value={wtime}
          bind:valid={wtimeOk}
          resetSignal={resetN}
          label="Run the pump for"
          units={['s']}
          hint={wtime > 60 ? 'Long runs can overflow the pot.' : 'How long one watering lasts.'}
        />
      </div>
      <div class="actions">
        <button class="btn primary" type="submit" disabled={!dirty || !freqOk || !wtimeOk || saving}>
          Save timing
        </button>
        <button class="btn" type="button" disabled={(!dirty && freqOk && wtimeOk) || saving} onclick={reset}>Discard changes</button>
      </div>
    </form>
  </details>
</article>

<style>
  .panel {
    display: grid;
    gap: 18px;
    padding: 24px 0;
    border-top: 1px solid var(--rule);
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 4px 16px;
  }
  .body {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 20px 32px;
  }
  @media (min-width: 760px) {
    .body {
      grid-template-columns: 210px minmax(0, 1fr);
    }
  }
  .now {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }
  .now-text {
    display: grid;
    gap: 4px;
    justify-items: start;
  }
  .value {
    font-size: 2.25rem;
    font-weight: 620;
    line-height: 1.05;
    letter-spacing: -0.02em;
  }
  small {
    margin-left: 2px;
    font-size: 1.0625rem;
    font-weight: 500;
    color: var(--ink-2);
  }
  .now-text .small {
    margin-bottom: 8px;
  }
  .hist {
    min-width: 0;
  }
  .timing summary {
    cursor: pointer;
    width: fit-content;
    font-weight: 560;
  }
  .timing form {
    display: grid;
    gap: 16px;
    margin-top: 14px;
  }
  .fields {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 32px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
