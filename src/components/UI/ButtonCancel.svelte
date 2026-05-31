<script lang="ts">
  export let icon = "i-iconoir-cancel";
  export let ariaLabel = "閉じる";
  export let title: string | undefined = undefined;
  export let iconClass = "h-5 w-5";
  export let colorClass = "color-text-tertiary hover:color-text-primary";

  $: resolvedTitle = title ?? ariaLabel;
</script>

<button
  on:click
  type="button"
  aria-label={ariaLabel}
  title={resolvedTitle}
  class="grid h-9 w-9 place-items-center rounded-md bg-transparent transition-all hover:bg-accent-primary/16 flex-shrink-0 {colorClass}"
>
  <div class="{iconClass} {icon}" />
</button>
