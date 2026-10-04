<script lang="ts">
  import { untrack } from 'svelte'
  import { U16_MAX } from '../types'

  interface Props {
    /** Seconds */
    value: number
    /** True when the current text is a whole number of seconds within [min, max] */
    valid?: boolean
    label: string
    hint?: string
    min?: number
    max?: number
    units?: ('s' | 'min' | 'h')[]
    /** Bump this number to make the field drop any half-typed text and show `value` again */
    resetSignal?: number
  }

  let {
    value = $bindable(),
    valid = $bindable(true),
    label,
    hint,
    min = 1,
    max = U16_MAX,
    units = ['s', 'min', 'h'],
    resetSignal = 0,
  }: Props = $props()

  const MULT = { s: 1, min: 60, h: 3600 } as const
  type Unit = keyof typeof MULT

  function bestUnit(secs: number): Unit {
    if (units.includes('h') && secs >= 3600 && secs % 3600 === 0) return 'h'
    if (units.includes('min') && secs >= 60 && secs % 60 === 0) return 'min'
    return units[0]
  }

  // svelte-ignore state_referenced_locally
  let unit = $state<Unit>(bestUnit(value))
  // svelte-ignore state_referenced_locally
  let text = $state(String(value / MULT[bestUnit(value)]))

  const seconds = $derived(Math.round(Number(text) * MULT[unit]))
  const textOk = $derived(text.trim() !== '' && Number.isFinite(Number(text)) && Number.isInteger(seconds))
  const inRange = $derived(textOk && seconds >= min && seconds <= max)
  const error = $derived(
    !textOk
      ? 'Enter a number.'
      : seconds < min
        ? `At least ${min} s.`
        : seconds > max
          ? `At most ${max.toLocaleString()} s (about ${(max / 3600).toFixed(1)} h).`
          : '',
  )

  $effect(() => {
    valid = inRange
  })

  // `lastPushed` is the last value this field wrote (or started with). A `value` that differs from it
  // came from outside (form reset, refresh after save) and replaces the text. Invalid text is left
  // alone while typing, so clearing the box doesn't snap back.
  // svelte-ignore state_referenced_locally
  let lastPushed = value

  // Push edits up
  $effect(() => {
    if (inRange && seconds !== untrack(() => value)) {
      lastPushed = seconds
      value = seconds
    }
  })

  // Discard: show the stored value again, even if the text is invalid
  $effect(() => {
    void resetSignal
    untrack(() => {
      lastPushed = value
      unit = bestUnit(value)
      text = String(value / MULT[unit])
    })
  })

  // Pull outside changes down
  $effect(() => {
    const v = value
    untrack(() => {
      if (v !== lastPushed) {
        lastPushed = v
        unit = bestUnit(v)
        text = String(v / MULT[unit])
      }
    })
  })

  const id = `dur-${Math.random().toString(36).slice(2, 9)}`
</script>

<div class="field">
  <label for={id}>{label}</label>
  <div class="row">
    <input
      {id}
      class="input num"
      type="number"
      inputmode="numeric"
      min="0"
      step="1"
      value={text}
      oninput={(e) => (text = e.currentTarget.value)}
      aria-invalid={!inRange}
      aria-describedby="{id}-msg"
    />
    {#if units.length > 1}
      <select class="input" bind:value={unit} aria-label="{label} unit">
        {#each units as u (u)}
          <option value={u}>{u}</option>
        {/each}
      </select>
    {:else}
      <span class="unit">{units[0]}</span>
    {/if}
  </div>
  <p id="{id}-msg" class="msg small" class:err={!!error}>{error || hint || ''}</p>
</div>

<style>
  .field {
    display: grid;
    gap: 6px;
  }
  label {
    font-weight: 560;
    font-size: 0.9375rem;
  }
  .row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  input {
    width: 7.5rem;
  }
  select {
    width: auto;
  }
  .unit {
    color: var(--ink-2);
  }
  .msg {
    min-height: 1.3em;
    color: var(--ink-2);
  }
  .msg.err {
    color: var(--bad);
  }
</style>
