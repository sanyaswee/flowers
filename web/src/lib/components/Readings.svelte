<script lang="ts">
  import { fmtNum } from '../format'
  import type { NodeEntry, NodeTelemetryEntry } from '../types'

  interface Props {
    node: NodeEntry
    latest: NodeTelemetryEntry | null | undefined
    dimmed?: boolean
  }
  let { node, latest, dimmed = false }: Props = $props()

  // Firmware treats a tank at or below 10 % as empty.
  const LOW_TANK = 10
  const level = $derived(latest?.water_tank_level ?? null)
</script>

<dl class="readings" class:dimmed>
  {#if node.water_tank}
    <div class="item">
      <dt>Water tank</dt>
      {#if node.water_tank_level}
        <dd>
          <span class="num value">{fmtNum(level)}<small>%</small></span>
          {#if level !== null}
            <span class="meter" aria-hidden="true"><span class="meter-fill" class:low={level <= LOW_TANK} style:width="{Math.min(100, Math.max(0, level))}%"></span></span>
            {#if level <= LOW_TANK}<span class="flag">Low</span>{/if}
          {/if}
        </dd>
      {:else}
        <dd>
          {#if latest?.water_tank_has_water === null || latest?.water_tank_has_water === undefined}
            <span class="value">–</span>
          {:else if latest.water_tank_has_water}
            <span class="value">Has water</span>
          {:else}
            <span class="value">Empty</span><span class="flag">Refill</span>
          {/if}
        </dd>
      {/if}
    </div>
  {/if}
  {#if node.temperature}
    <div class="item">
      <dt>Temperature</dt>
      <dd><span class="num value">{fmtNum(latest?.temperature, 1)}<small>°C</small></span></dd>
    </div>
  {/if}
  {#if node.humidity}
    <div class="item">
      <dt>Humidity</dt>
      <dd><span class="num value">{fmtNum(latest?.humidity)}<small>%</small></span></dd>
    </div>
  {/if}
  {#if node.pressure}
    <div class="item">
      <dt>Pressure</dt>
      <dd><span class="num value">{fmtNum(latest?.pressure)}<small>hPa</small></span></dd>
    </div>
  {/if}
  {#if node.light}
    <div class="item">
      <dt>Light</dt>
      <dd><span class="num value">{fmtNum(latest?.light_intensity)}<small>lx</small></span></dd>
    </div>
  {/if}
</dl>

<style>
  .readings {
    display: flex;
    flex-wrap: wrap;
    gap: 12px 32px;
    margin: 0;
  }
  .readings.dimmed {
    opacity: 0.6;
  }
  .item {
    display: grid;
    gap: 2px;
  }
  dt {
    color: var(--ink-2);
    font-size: 0.875rem;
  }
  dd {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
  }
  .value {
    font-size: 1.375rem;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  small {
    margin-left: 3px;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--ink-2);
  }
  .meter {
    width: 56px;
    height: 8px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--c-tank) 22%, var(--plate));
    overflow: hidden;
  }
  .meter-fill {
    display: block;
    height: 100%;
    border-radius: 4px;
    background: var(--c-tank);
  }
  .meter-fill.low {
    background: var(--warn);
  }
  .flag {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--warn);
  }
</style>
