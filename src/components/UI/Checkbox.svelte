<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let value: boolean;
  export let disabled = false;

  const dispather = createEventDispatcher<{ update: { value: boolean } }>();
  $: {
    dispather("update", { value });
  }
</script>

<input
  type="checkbox"
  checked={value}
  {disabled}
  on:change={(e) => {
    if (disabled) return;
    value = e.currentTarget.checked;
  }}
  class="hidden"
/>
{#if value}
  <div
    class="i-material-symbols-check-box-rounded color-border-button w-6 h-6 {disabled ? 'opacity-50 cursor-not-allowed' : ''}"
  />
{:else}
  <div
    class="i-material-symbols-check-box-outline-blank color-border-button w-6 h-6 {disabled ? 'opacity-50 cursor-not-allowed' : ''}"
  />
{/if}
