<script lang="ts">
  /** A soil probe: a tube filled to the moisture percentage. The track is a lighter step of the fill's blue. */
  interface Props {
    /** 0..100, or null when there is no reading */
    value: number | null
    height?: number
    dimmed?: boolean
  }
  let { value, height = 128, dimmed = false }: Props = $props()

  const W = 36
  const R = 10
  const pad = 2
  const innerH = $derived(height - pad * 2)
  const pct = $derived(value === null ? 0 : Math.min(100, Math.max(0, value)))
  const fillH = $derived((innerH * pct) / 100)
  const clipId = `probe-${Math.random().toString(36).slice(2, 9)}`
</script>

<svg
  class="probe"
  class:dimmed
  width={W}
  height={height}
  viewBox="0 0 {W} {height}"
  role="img"
  aria-label={value === null ? 'No moisture reading' : `Soil moisture ${Math.round(pct)} percent`}
>
  <defs>
    <clipPath id={clipId}>
      <rect x={pad} y={pad} width={W - pad * 2} height={innerH} rx={R - pad} />
    </clipPath>
  </defs>
  <rect x={pad} y={pad} width={W - pad * 2} height={innerH} rx={R - pad} class="track" />
  <g clip-path="url(#{clipId})">
    <rect x={pad} y={pad + innerH - fillH} width={W - pad * 2} height={fillH} class="fill" />
  </g>
  {#each [25, 50, 75] as tick (tick)}
    <line x1={W - pad - 7} x2={W - pad} y1={pad + innerH * (1 - tick / 100)} y2={pad + innerH * (1 - tick / 100)} class="tick" />
  {/each}
</svg>

<style>
  .probe {
    display: block;
    flex: none;
  }
  .probe.dimmed {
    opacity: 0.55;
  }
  .track {
    fill: var(--water-track);
  }
  .fill {
    fill: var(--c-moisture);
    transition:
      y 0.4s ease,
      height 0.4s ease;
  }
  .tick {
    stroke: var(--plate);
    stroke-width: 1.5;
    opacity: 0.9;
  }
</style>
