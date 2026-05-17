<script lang="ts">
  export let noBorder = false;
  export let label: string;
</script>

<div
  class="p-4 border-solid border-border-primary whitespace-nowrap text-body2 text-text-primary {noBorder
    ? 'border-0px'
    : 'border-t-1px'}"
>
  {label}
</div>
<div
  class="p-4 border-solid border-border-primary text-body2 text-text-link {noBorder
    ? 'border-0px'
    : 'border-t-1px'}"
>
  <slot />
</div>
