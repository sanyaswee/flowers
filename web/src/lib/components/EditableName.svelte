<script lang="ts">
  import { tick } from 'svelte'

  interface Props {
    /** Name as shown (already falls back to the id) */
    display: string
    /** Raw custom name, '' when unnamed */
    value: string
    level?: 'h1' | 'h2' | 'h3'
    what: string
    onsave: (name: string) => Promise<boolean>
  }
  let { display, value, level = 'h2', what, onsave }: Props = $props()

  let editing = $state(false)
  let draft = $state('')
  let saving = $state(false)
  let input = $state<HTMLInputElement>()

  async function start() {
    draft = value
    editing = true
    await tick()
    input?.focus()
    input?.select()
  }

  async function save() {
    if (saving) return
    if (draft.trim() === value.trim()) {
      editing = false
      return
    }
    saving = true
    const ok = await onsave(draft)
    saving = false
    if (ok) editing = false
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') editing = false
  }
</script>

{#if editing}
  <form
    class="edit"
    onsubmit={(e) => {
      e.preventDefault()
      void save()
    }}
  >
    <input
      bind:this={input}
      bind:value={draft}
      class="input"
      aria-label="Name of {what}"
      placeholder={display}
      maxlength="60"
      onkeydown={onKey}
    />
    <button class="btn primary" type="submit" disabled={saving}>Save</button>
    <button class="btn" type="button" onclick={() => (editing = false)}>Cancel</button>
  </form>
{:else}
  <div class="show">
    <svelte:element this={level} class="name">{display}</svelte:element>
    <button class="btn quiet rename" type="button" onclick={start} aria-label="Rename {what}">
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
        <path d="M2 14h3l8-8-3-3-8 8v3Zm9-11 3 3" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
      </svg>
      Rename
    </button>
  </div>
{/if}

<style>
  .show,
  .edit {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 12px;
  }
  .name {
    overflow-wrap: anywhere;
  }
  .rename {
    min-height: 32px;
    padding: 0 10px;
    color: var(--ink-2);
    font-weight: 500;
  }
  .edit .input {
    min-width: 12rem;
    flex: 1 1 14rem;
    max-width: 24rem;
  }
</style>
