<script lang="ts">
  import { fly } from "svelte/transition";

  export let appendClass: string;
  export let back: boolean = false;
  export let show: boolean;
</script>

{#if show}
  <div
    class="absolute p-(x-2 y-2) {appendClass} top--1 bottom--1 flex items-center justify-center z-10"
    class:rotate-180={back}
    transition:fly={{ x: 10, duration: 150 }}
  >
    <button
      class="glass rounded-full p-1.5 hover:bg-bg-button-hover transition-all flex items-center justify-center shadow-sm"
      on:click|stopPropagation
    >
      <div
        class="i-material-symbols-arrow-forward-ios-rounded w-4 h-4 color-text-primary"
      />
    </button>
  </div>
{/if}
