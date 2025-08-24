<script lang="ts">
  export let label: string;
</script>

<div
  class="grid grid-cols-[min-content_1fr] border-b border-solid border-primary last:border-b-0"
>
  <div class="p-4 whitespace-nowrap text-body2 text-primary">{label}</div>
  <div class="p-4 text-body2 text-link">
    <slot />
  </div>
</div>
