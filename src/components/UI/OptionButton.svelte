<script lang="ts">
  export let selected: boolean = false;
  export let text: string;
  export let withBorderTop = false;
  export let showIcon = false;
</script>

<button
  on:click
  class="w-full text-left px-4 py-1.5 text-sm text-text-primary bg-transparent hover:bg-accent-primary/16 transition-colors flex items-center {withBorderTop &&
    'border-t-1px border-solid border-border-primary'}"
>
  {#if showIcon}
    <div
      class="h-5 w-5 color-text-primary"
      class:i-material-symbols-check-small-rounded={selected}
    />
  {/if}
  <div class="font-normal whitespace-nowrap">
    {text}
  </div>
</button>
