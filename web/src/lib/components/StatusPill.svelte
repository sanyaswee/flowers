<script lang="ts">
  import type { NodeStatus } from '../format'

  interface Props {
    status: NodeStatus
    /** e.g. "seen 12 s ago" */
    detail?: string
  }
  let { status, detail }: Props = $props()
</script>

<span class="pill" class:offline={status === 'offline'}>
  <!-- Filled dot = online, hollow ring = offline, so state never relies on color alone -->
  <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
    {#if status === 'online'}
      <circle cx="6" cy="6" r="5" class="dot" />
    {:else}
      <circle cx="6" cy="6" r="4" class="ring" />
    {/if}
  </svg>
  <strong>{status === 'online' ? 'Online' : 'Offline'}</strong>
  {#if detail}<span class="detail">{detail}</span>{/if}
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 0.9375rem;
  }
  strong {
    font-weight: 600;
    color: var(--good);
  }
  .offline strong {
    color: var(--ink-2);
  }
  .dot {
    fill: var(--good);
  }
  .ring {
    fill: none;
    stroke: var(--ink-3);
    stroke-width: 2;
  }
  .detail {
    color: var(--ink-2);
  }
</style>
