<script lang="ts">
  import { createEventDispatcher, onMount, tick } from "svelte";

  export let label = "";
  export let value: string;
  export let placeholder: string = "";
  export let autofocus = false;
  export let disabled = false;

  const dispatcher = createEventDispatcher<{ update: { value: string } }>();

  let input: HTMLInputElement | null = null;

  onMount(async () => {
    if (!autofocus) {
      return;
    }
    await tick();
    input?.focus();
  });
</script>

<label>
  {#if label}
    <div class="text-text-primary text-body font-medium mb-1">{label}</div>
  {/if}
  <div
    class="w-full border-2px border-solid border-border-primary rounded transition-all {disabled ? '' : 'focus-within:border-accent-accent'}"
  >
    <input
      bind:this={input}
      bind:value
      type="text"
      on:input={(e) => dispatcher("update", { value: e.currentTarget.value })}
      {placeholder}
      {disabled}
      class="w-full border-none outline-none rounded bg-bg-primary p-x-3 p-y-1 text-input text-text-primary transition-all placeholder-ui-tertiary disabled:opacity-50 disabled:cursor-not-allowed"
    />
  </div>
</label>
