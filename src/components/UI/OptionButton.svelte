<script lang="ts">
  export let selected: boolean = false;
  export let text: string;
  export let withBorderTop = true;
  export let showIcon = false;
</script>

<button
  on:click
  class="p-x-4 p-y-2 flex items-center bg-transparent transition-all hover:bg-bg-button-hover w-full {withBorderTop &&
    'border-t-1px border-solid border-border-primary'}"
>
  {#if showIcon}
    <div
      class="h-5 w-5 color-text-primary"
      class:i-material-symbols-check-small-rounded={selected}
    />
  {/if}
  <div class="text-text-primary text-body2 font-bold whitespace-nowrap">
    {text}
  </div>
</button>
