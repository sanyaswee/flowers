// Time helpers.
//
// The backend stores and returns UTC timestamps without an offset ("NaiveDateTime"), and
// the browser clock can differ from the server clock. `clock.offset` is the estimated
// difference (server - browser), learned from the HTTP Date header of every API response,
// so "last seen" and "online" stay right even if this device's clock is off.

export const clock = $state({ now: Date.now(), offset: 0 })

if (typeof window !== 'undefined') {
  setInterval(() => {
    clock.now = Date.now()
  }, 1000)
}

/** Current server time in ms. Reactive: reads the ticking clock. */
export function serverNow(): number {
  return clock.now + clock.offset
}

/** Current server time in ms, non-reactive. */
export function serverNowStatic(): number {
  return Date.now() + clock.offset
}

const NAIVE = /^(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}:\d{2})(?:\.(\d+))?(Z|[+-]\d{2}:?\d{2})?$/

/** Parse a backend timestamp (UTC, usually with no zone suffix) to epoch ms. */
export function parseServerTime(s: string): number {
  const m = NAIVE.exec(s)
  if (!m) return Date.parse(s)
  const ms = (m[3] ?? '0').slice(0, 3).padEnd(3, '0')
  return Date.parse(`${m[1]}T${m[2]}.${ms}${m[4] ?? 'Z'}`)
}

/** Format epoch ms for the `start` / `end` query params: UTC, no zone suffix (chrono rejects "Z"). */
export function toNaiveUtc(ms: number): string {
  return new Date(ms).toISOString().slice(0, 19)
}
