<script lang="ts">
  export let title: string | undefined = undefined;
</script>

<div class="p-4 border-(border-primary solid ~) rounded space-y-2">
  {#if title}
    <div class="text-(text-primary h3) font-medium">{title}</div>
  {/if}
  <div class="text-(text-tertiary body)">
    <slot />
  </div>
</div>
