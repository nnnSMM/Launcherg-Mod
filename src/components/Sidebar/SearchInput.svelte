<script lang="ts">
  export let value: string;
  export let placeholder: string = "";
  export let ariaLabel: string | undefined = undefined;

  let input: HTMLInputElement | null = null;
  const clearLabel =
    "\u691c\u7d22\u6587\u5b57\u5217\u3092\u30af\u30ea\u30a2";
</script>

<svelte:window
  on:keydown={(e) => {
    if (e.key === "/") {
      const active = document.activeElement;
      if (
        active &&
        (active.tagName === "INPUT" || active.tagName === "TEXTAREA")
      ) {
        return;
      }
      setTimeout(() => {
        if (input) {
          input.focus();
        }
      }, 20);
    }
  }}
/>
<div
  class="border-2px border-solid border-transparent focus-within:border-accent-accent rounded transition-all min-w-0 w-full"
>
  <div
    class="group w-full flex items-center gap-2 px-2 py-1 border border-border-primary rounded bg-transparent hover:bg-white/5 hover:border-border-button-hover focus-within:border-accent-accent focus-within:bg-transparent transition-all relative min-w-0"
  >
    <div class="w-5 h-5 i-material-symbols-search color-text-primary shrink-0" />
    <input
      bind:this={input}
      bind:value
      {placeholder}
      aria-label={ariaLabel || placeholder}
      type="search"
      class="w-full min-w-0 flex-1 text-body2 text-text-primary bg-transparent placeholder-text-placeholder transition-all outline-none"
    />
    {#if value !== ""}
      <button
        type="button"
        on:click={() => (value = "")}
        aria-label={clearLabel}
        title={clearLabel}
        class="absolute right-2 w-5 h-5 i-material-symbols-cancel-outline-rounded color-text-primary"
      />
    {/if}
  </div>
</div>
