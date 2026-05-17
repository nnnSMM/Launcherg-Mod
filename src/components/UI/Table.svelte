<script lang="ts">
  export let title: string;
  export let rows: {
    label: string;
    value: string;
    component?: any;
  }[];
</script>

<div class="border border-solid border-border-primary rounded-xl">
  <div class="bg-black/20 p-4 rounded-t-xl">
    <div class="text-body text-text-primary">{title}</div>
  </div>
  <div class="grid grid-cols-[min-content_1fr]">
    {#each rows as row (row.label)}
      <div
        class="p-4 border-t-1px border-solid border-border-primary whitespace-nowrap text-body2 text-text-primary"
      >
        {row.label}
      </div>
      {#if row.component}
        <div class="p-4 border-t-1px border-solid border-border-primary">
          <svelte:component this={row.component} value={row.value} />
        </div>
      {:else}
        <div
          class="p-4 border-t-1px border-solid border-border-primary text-body2 text-text-primary"
        >
          {row.value}
        </div>
      {/if}
    {/each}
  </div>
</div>
