<script lang="ts">
  export let icon = "i-iconoir-cancel";
  export let ariaLabel = "閉じる";
  export let title = ariaLabel;
</script>

<button
  on:click
  type="button"
  aria-label={ariaLabel}
  {title}
  class="ml-auto w-5 h-5 {icon} color-text-tertiary hover:color-text-primary transition-all flex-shrink-0"
/>
