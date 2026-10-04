<script lang="ts">
  import { untrack } from 'svelte'
  import { saveNodeSettings } from '../actions'
  import type { NodeEntry } from '../types'
  import DurationField from './DurationField.svelte'

  interface Props {
    node: NodeEntry
  }
  let { node }: Props = $props()

  // The draft follows the server until the user edits it.
  // svelte-ignore state_referenced_locally
  let report = $state(node.telemetry_report_freq)
  // svelte-ignore state_referenced_locally
  let light = $state(node.light_m_freq)
  // svelte-ignore state_referenced_locally
  let bmpe = $state(node.bmpe_m_freq)
  let reportOk = $state(true)
  let lightOk = $state(true)
  let bmpeOk = $state(true)
  let saving = $state(false)
  let resetN = $state(0)

  const hasAir = $derived(node.temperature || node.humidity || node.pressure)
  const dirty = $derived(
    report !== node.telemetry_report_freq || light !== node.light_m_freq || bmpe !== node.bmpe_m_freq,
  )
  const valid = $derived(reportOk && (!node.light || lightOk) && (!hasAir || bmpeOk))

  $effect(() => {
    const r = node.telemetry_report_freq
    const l = node.light_m_freq
    const b = node.bmpe_m_freq
    untrack(() => {
      if (!dirty) {
        report = r
        light = l
        bmpe = b
      }
    })
  })

  async function save() {
    saving = true
    await saveNodeSettings(node, {
      telemetry_packet_creation_freq_s: report,
      light_intensity_s: light,
      bmpe_s: bmpe,
    })
    saving = false
  }

  function reset() {
    report = node.telemetry_report_freq
    light = node.light_m_freq
    bmpe = node.bmpe_m_freq
    resetN++
  }
</script>

<form
  onsubmit={(e) => {
    e.preventDefault()
    void save()
  }}
>
  <div class="fields">
    <DurationField
      bind:value={report}
      bind:valid={reportOk}
      resetSignal={resetN}
      label="Send readings every"
      hint="How often the node reports to the server."
    />
    {#if hasAir}
      <DurationField
        bind:value={bmpe}
        bind:valid={bmpeOk}
        resetSignal={resetN}
        label="Read air sensor every"
        hint="Temperature, humidity and pressure."
      />
    {/if}
    {#if node.light}
      <DurationField
        bind:value={light}
        bind:valid={lightOk}
        resetSignal={resetN}
        label="Read light sensor every"
        hint="How often brightness is measured."
      />
    {/if}
  </div>
  <p class="small muted">
    Changes are saved on the server and sent to the node. If the node is offline, it picks them up the next time it boots.
  </p>
  <div class="actions">
    <button class="btn primary" type="submit" disabled={!dirty || !valid || saving}>Save settings</button>
    <button class="btn" type="button" disabled={(!dirty && valid) || saving} onclick={reset}>Discard changes</button>
  </div>
</form>

<style>
  form {
    display: grid;
    gap: 16px;
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
