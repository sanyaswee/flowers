<script lang="ts">
  import { dismiss, toasts } from '../toast.svelte'
</script>

<div class="toasts" aria-live="polite" aria-atomic="false">
  {#each toasts.items as t (t.id)}
    <div class="toast" class:error={t.kind === 'error'} role={t.kind === 'error' ? 'alert' : 'status'}>
      <div>
        <strong>{t.title}</strong>
        {#if t.detail}<p class="small">{t.detail}</p>{/if}
      </div>
      <button class="close" type="button" aria-label="Dismiss" onclick={() => dismiss(t.id)}>
        <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
          <path d="m2 2 10 10M12 2 2 12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    inset: auto 0 16px 0;
    display: grid;
    justify-items: center;
    gap: 8px;
    padding: 0 var(--gutter);
    pointer-events: none;
    z-index: 10;
  }
  .toast {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    max-width: min(30rem, 100%);
    padding: 10px 12px 10px 16px;
    border: 1px solid var(--rule);
    border-left: 4px solid var(--good);
    border-radius: var(--radius);
    background: var(--plate);
    box-shadow: 0 4px 16px rgb(0 0 0 / 0.16);
    pointer-events: auto;
  }
  .toast.error {
    border-left-color: var(--bad);
  }
  .toast p {
    color: var(--ink-2);
  }
  .close {
    flex: none;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--ink-2);
    cursor: pointer;
  }
  .close:hover {
    background: var(--wash);
  }
</style>
