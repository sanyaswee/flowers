<script lang="ts">
  import { downsample, niceScale, splitOnGaps, timeTicks, type Pt } from '../chartData'

  interface Props {
    /** Ascending by time */
    points: Pt[]
    /** CSS color, normally a var(--c-*) token */
    color: string
    /** What is plotted, for the accessible name and tooltip */
    label: string
    unit: string
    decimals?: number
    /** Fixed y range, e.g. [0, 100] for percentages */
    domain?: [number, number]
    /** Auto range, but always include zero */
    zeroBase?: boolean
    xDomain: [number, number]
    height?: number
    loading?: boolean
  }

  let {
    points,
    color,
    label,
    unit,
    decimals = 0,
    domain,
    zeroBase = false,
    xDomain,
    height = 168,
    loading = false,
  }: Props = $props()

  const M = { l: 44, r: 62, t: 12, b: 24 }

  let width = $state(640)
  const plotW = $derived(Math.max(40, width - M.l - M.r))
  const plotH = $derived(height - M.t - M.b)

  const segments = $derived(
    splitOnGaps(points).map((s) => downsample(s, Math.max(40, Math.floor(plotW / 2)))),
  )
  const flat = $derived(segments.flat())

  const yScale = $derived.by(() => {
    const target = Math.max(2, Math.floor(plotH / 40))
    if (domain) return niceScale(domain[0], domain[1], target)
    if (!points.length) return niceScale(0, 1, target)
    let lo = Infinity
    let hi = -Infinity
    for (const p of points) {
      if (p.v < lo) lo = p.v
      if (p.v > hi) hi = p.v
    }
    if (zeroBase) lo = Math.min(0, lo)
    if (lo === hi) {
      const pad = Math.max(1, Math.abs(lo) * 0.05)
      lo -= pad
      hi += pad
    } else {
      const pad = (hi - lo) * 0.08
      hi += pad
      if (!zeroBase || lo < 0) lo -= pad
    }
    return niceScale(lo, hi, target)
  })

  const x = (t: number) => M.l + ((t - xDomain[0]) / (xDomain[1] - xDomain[0] || 1)) * plotW
  const y = (v: number) => M.t + (1 - (v - yScale.min) / (yScale.max - yScale.min || 1)) * plotH

  const xTicks = $derived(timeTicks(xDomain[0], xDomain[1], Math.max(2, Math.floor(plotW / 96))))

  const linePaths = $derived(
    segments
      .filter((s) => s.length > 1)
      .map((s) => s.map((p, i) => `${i ? 'L' : 'M'}${x(p.t).toFixed(1)} ${y(p.v).toFixed(1)}`).join('')),
  )
  const areaPaths = $derived(
    segments
      .filter((s) => s.length > 1)
      .map((s) => {
        const line = s.map((p, i) => `${i ? 'L' : 'M'}${x(p.t).toFixed(1)} ${y(p.v).toFixed(1)}`).join('')
        const base = y(yScale.min).toFixed(1)
        return `${line}L${x(s[s.length - 1].t).toFixed(1)} ${base}L${x(s[0].t).toFixed(1)} ${base}Z`
      }),
  )
  const lonelyDots = $derived(segments.filter((s) => s.length === 1).map((s) => s[0]))

  const last = $derived(flat.length ? flat[flat.length - 1] : null)

  let hoverIndex = $state<number | null>(null)
  const hover = $derived(hoverIndex === null ? null : (flat[hoverIndex] ?? null))

  function nearest(t: number): number {
    let lo = 0
    let hi = flat.length - 1
    while (lo < hi) {
      const mid = (lo + hi) >> 1
      if (flat[mid].t < t) lo = mid + 1
      else hi = mid
    }
    if (lo > 0 && Math.abs(flat[lo - 1].t - t) <= Math.abs(flat[lo].t - t)) return lo - 1
    return lo
  }

  function onMove(e: PointerEvent) {
    if (!flat.length) return
    const rect = (e.currentTarget as SVGElement).getBoundingClientRect()
    const px = e.clientX - rect.left
    const t = xDomain[0] + ((px - M.l) / plotW) * (xDomain[1] - xDomain[0])
    hoverIndex = nearest(t)
  }

  function onKey(e: KeyboardEvent) {
    if (!flat.length) return
    if (e.key === 'ArrowLeft') hoverIndex = Math.max(0, (hoverIndex ?? flat.length) - 1)
    else if (e.key === 'ArrowRight') hoverIndex = Math.min(flat.length - 1, (hoverIndex ?? -1) + 1)
    else if (e.key === 'Escape') hoverIndex = null
    else return
    e.preventDefault()
  }

  const fmt = (v: number) => v.toFixed(decimals)
  const fmtTime = (t: number) =>
    new Date(t).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' })

  const summary = $derived(
    last ? `${label}, latest ${fmt(last.v)} ${unit}` : `${label}, no readings in this period`,
  )
</script>

<div class="chart" class:loading bind:clientWidth={width}>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <svg
    {width}
    {height}
    viewBox="0 0 {width} {height}"
    role="img"
    aria-label={summary}
    tabindex="0"
    onpointermove={onMove}
    onpointerleave={() => (hoverIndex = null)}
    onkeydown={onKey}
    onblur={() => (hoverIndex = null)}
  >
    {#each yScale.ticks as tick (tick)}
      <line class="grid" x1={M.l} x2={M.l + plotW} y1={y(tick)} y2={y(tick)} />
      <text class="axis" x={M.l - 8} y={y(tick)} text-anchor="end" dominant-baseline="middle">{fmt(tick)}</text>
    {/each}
    {#each xTicks as tick (tick.t)}
      <text class="axis" x={x(tick.t)} y={height - 6} text-anchor="middle">{tick.label}</text>
    {/each}

    {#each areaPaths as d, i (i)}
      <path {d} class="area" style:fill={color} />
    {/each}
    {#each linePaths as d, i (i)}
      <path {d} class="line" style:stroke={color} />
    {/each}
    {#each lonelyDots as p (p.t)}
      <circle cx={x(p.t)} cy={y(p.v)} r="4" class="dot" style:fill={color} />
    {/each}

    {#if !flat.length}
      <text class="empty" x={M.l + plotW / 2} y={M.t + plotH / 2} text-anchor="middle">No readings in this period</text>
    {/if}

    {#if last && !hover}
      <circle cx={x(last.t)} cy={y(last.v)} r="4" class="dot" style:fill={color} />
      <text class="endlabel" x={M.l + plotW + 10} y={y(last.v)} dominant-baseline="middle">
        {fmt(last.v)} {unit}
      </text>
    {/if}

    {#if hover}
      <line class="cross" x1={x(hover.t)} x2={x(hover.t)} y1={M.t} y2={M.t + plotH} />
      <circle cx={x(hover.t)} cy={y(hover.v)} r="4" class="dot" style:fill={color} />
    {/if}
  </svg>

  {#if hover}
    {@const px = x(hover.t)}
    <div
      class="tip"
      style:left="{px < width / 2 ? px + 12 : px - 12}px"
      style:transform={px < width / 2 ? 'none' : 'translateX(-100%)'}
    >
      <span class="key" style:background={color}></span>
      <strong class="num">{fmt(hover.v)} {unit}</strong>
      <span class="when">{fmtTime(hover.t)}</span>
    </div>
  {/if}
</div>

<style>
  .chart {
    position: relative;
    width: 100%;
    transition: opacity 0.15s;
  }
  .chart.loading {
    opacity: 0.55;
  }
  svg {
    display: block;
    touch-action: pan-y;
    overflow: visible;
  }
  .grid {
    stroke: var(--grid);
    stroke-width: 1;
  }
  .axis {
    fill: var(--ink-3);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }
  .line {
    fill: none;
    stroke-width: 2;
    stroke-linejoin: round;
    stroke-linecap: round;
  }
  .area {
    opacity: 0.1;
  }
  .dot {
    stroke: var(--plate);
    stroke-width: 2;
  }
  .cross {
    stroke: var(--ink-3);
    stroke-width: 1;
  }
  .endlabel {
    fill: var(--ink-2);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .empty {
    fill: var(--ink-3);
    font-size: 13px;
  }
  .tip {
    position: absolute;
    top: 6px;
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 6px 10px;
    border: 1px solid var(--rule);
    border-radius: 8px;
    background: var(--plate);
    box-shadow: 0 2px 8px rgb(0 0 0 / 0.12);
    font-size: 0.8125rem;
    pointer-events: none;
    white-space: nowrap;
  }
  .key {
    display: inline-block;
    width: 12px;
    height: 3px;
    border-radius: 2px;
    align-self: center;
  }
  .when {
    color: var(--ink-2);
  }
</style>
