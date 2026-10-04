<script lang="ts">
  interface Props {
    checked: boolean
    label: string
    disabled?: boolean
    onchange: (next: boolean) => void
  }
  let { checked, label, disabled = false, onchange }: Props = $props()
</script>

<button
  type="button"
  class="switch"
  role="switch"
  aria-checked={checked}
  {disabled}
  onclick={() => onchange(!checked)}
>
  <span class="track" aria-hidden="true"><span class="thumb"></span></span>
  <span class="text">{label}</span>
</button>

<style>
  .switch {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    min-height: 40px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--ink);
    font: inherit;
    cursor: pointer;
  }
  .switch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .track {
    position: relative;
    width: 40px;
    height: 24px;
    border-radius: 12px;
    background: var(--rule);
    transition: background-color 0.15s;
    flex: none;
  }
  .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--plate);
    transition: transform 0.15s;
  }
  .switch[aria-checked='true'] .track {
    background: var(--water-strong);
  }
  .switch[aria-checked='true'] .thumb {
    transform: translateX(16px);
  }
</style>
