<script module lang="ts">
  export const RANGES = {
    '1h': { label: '1 hour', ms: 3_600_000 },
    '6h': { label: '6 hours', ms: 6 * 3_600_000 },
    '24h': { label: '24 hours', ms: 24 * 3_600_000 },
    '7d': { label: '7 days', ms: 7 * 24 * 3_600_000 },
  } as const

  export type RangeKey = keyof typeof RANGES
</script>

<script lang="ts">
  interface Props {
    value: RangeKey
  }
  let { value = $bindable() }: Props = $props()
</script>

<div class="range" role="radiogroup" aria-label="History range">
  {#each Object.entries(RANGES) as [key, r] (key)}
    <button
      type="button"
      role="radio"
      aria-checked={value === key}
      class:on={value === key}
      onclick={() => (value = key as RangeKey)}
    >
      {r.label}
    </button>
  {/each}
</div>

<style>
  .range {
    display: inline-flex;
    padding: 3px;
    border: 1px solid var(--rule);
    border-radius: 12px;
    background: var(--plate);
  }
  button {
    min-height: 34px;
    padding: 0 14px;
    border: 0;
    border-radius: 9px;
    background: none;
    color: var(--ink-2);
    font: inherit;
    font-weight: 560;
    cursor: pointer;
  }
  button:hover:not(.on) {
    background: var(--wash);
  }
  button.on {
    background: var(--water-strong);
    color: #fff;
  }
</style>
