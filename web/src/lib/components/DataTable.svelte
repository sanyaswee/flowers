<script lang="ts">
  import type { Pt } from '../chartData'

  interface Props {
    label: string
    unit: string
    decimals?: number
    /** Ascending by time */
    points: Pt[]
    rows?: number
  }
  let { label, unit, decimals = 0, points, rows = 12 }: Props = $props()

  const recent = $derived(points.slice(-rows).reverse())
</script>

{#if points.length}
  <details>
    <summary>Show recent readings</summary>
    <table>
      <caption class="sr-only">Latest {recent.length} readings of {label}</caption>
      <thead>
        <tr><th scope="col">Time</th><th scope="col">{label} ({unit})</th></tr>
      </thead>
      <tbody>
        {#each recent as p (p.t)}
          <tr>
            <td>{new Date(p.t).toLocaleString([], { dateStyle: 'medium', timeStyle: 'medium' })}</td>
            <td class="num">{p.v.toFixed(decimals)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </details>
{/if}

<style>
  details {
    margin-top: 8px;
    font-size: 0.875rem;
  }
  summary {
    cursor: pointer;
    color: var(--ink-2);
    width: fit-content;
  }
  table {
    margin-top: 8px;
    border-collapse: collapse;
    width: 100%;
    max-width: 28rem;
  }
  th,
  td {
    padding: 4px 8px 4px 0;
    text-align: left;
    border-bottom: 1px solid var(--grid);
  }
  th {
    font-weight: 600;
  }
  td.num {
    font-weight: 560;
  }
</style>
