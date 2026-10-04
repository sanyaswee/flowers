export interface Pt {
  t: number
  v: number
}

export function median(xs: number[]): number {
  if (!xs.length) return 0
  const s = [...xs].sort((a, b) => a - b)
  const m = s.length >> 1
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2
}

/** Split an ascending series wherever the gap is much longer than usual (node offline, reboot). */
export function splitOnGaps(pts: Pt[], factor = 4, minGapMs = 120_000): Pt[][] {
  if (pts.length < 2) return pts.length ? [pts] : []
  const dts: number[] = []
  for (let i = 1; i < pts.length; i++) dts.push(pts[i].t - pts[i - 1].t)
  const limit = Math.max(median(dts) * factor, minGapMs)
  const out: Pt[][] = [[pts[0]]]
  for (let i = 1; i < pts.length; i++) {
    if (pts[i].t - pts[i - 1].t > limit) out.push([pts[i]])
    else out[out.length - 1].push(pts[i])
  }
  return out
}

/** Average into at most `max` points. Keeps the first point's time of each bucket's mean. */
export function downsample(seg: Pt[], max: number): Pt[] {
  if (seg.length <= max || max < 2) return seg
  const size = Math.ceil(seg.length / max)
  const out: Pt[] = []
  for (let i = 0; i < seg.length; i += size) {
    const chunk = seg.slice(i, i + size)
    let t = 0
    let v = 0
    for (const p of chunk) {
      t += p.t
      v += p.v
    }
    out.push({ t: t / chunk.length, v: v / chunk.length })
  }
  return out
}

export function niceScale(lo: number, hi: number, target: number) {
  const span = hi - lo || 1
  const step0 = span / Math.max(1, target)
  const mag = 10 ** Math.floor(Math.log10(step0))
  const norm = step0 / mag
  const step = (norm < 1.5 ? 1 : norm < 3 ? 2 : norm < 7 ? 5 : 10) * mag
  const min = Math.floor(lo / step) * step
  const max = Math.ceil(hi / step) * step
  const ticks: number[] = []
  for (let v = min; v <= max + step / 2; v += step) ticks.push(Number(v.toFixed(10)))
  return { min, max, ticks }
}

const MIN = 60_000
const HOUR = 60 * MIN
const DAY = 24 * HOUR
const STEPS = [MIN, 5 * MIN, 10 * MIN, 15 * MIN, 30 * MIN, HOUR, 2 * HOUR, 3 * HOUR, 6 * HOUR, 12 * HOUR, DAY, 2 * DAY]

/** Ticks on round local-time boundaries. */
export function timeTicks(start: number, end: number, maxTicks: number): { t: number; label: string }[] {
  const span = end - start
  const step = STEPS.find((s) => span / s <= maxTicks) ?? STEPS[STEPS.length - 1]
  const off = new Date(start).getTimezoneOffset() * 60_000
  const first = Math.ceil((start - off) / step) * step + off
  const multiDay = span > 30 * HOUR
  const out: { t: number; label: string }[] = []
  for (let t = first; t <= end; t += step) {
    const d = new Date(t)
    let label: string
    if (step >= DAY) label = d.toLocaleDateString([], { weekday: 'short', day: 'numeric' })
    else if (multiDay)
      label = d.toLocaleString([], { weekday: 'short', hour: '2-digit', minute: '2-digit' })
    else label = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    out.push({ t, label })
  }
  return out
}
