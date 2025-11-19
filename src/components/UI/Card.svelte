<script lang="ts">
  export let title: string | undefined = undefined;
  export let className: string = "";
</script>

<div class="p-6 rounded-xl glass-card space-y-3 group {className}">
  {#if title}
    <div
      class="text-(text-primary h3) font-bold tracking-tight group-hover:text-accent-accent transition-colors duration-200"
    >
      {title}
    </div>
  {/if}
  <div class="text-(text-secondary body) leading-relaxed">
    <slot />
  </div>
</div>
