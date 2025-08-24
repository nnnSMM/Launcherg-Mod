<script lang="ts">
  export let back: boolean = false;
  export let disabled: boolean = false;
</script>

<div class:rotate-180={back} class:opacity-30={disabled}>
  <button
    class="bg-transparent transition-all hover:bg-bg-button-hover rounded-full p-2"
    on:click|stopPropagation
    {disabled}
  >
    <div
      class="i-material-symbols-arrow-forward-ios-rounded w-5 h-5 color-text-primary"
    />
  </button>
</div>
