<script lang="ts">
  import {
    ATTRIBUTE_LABELS,
    type Attribute,
  } from "@/components/Sidebar/searchAttributes";

  export let attribute: Attribute;

  $: label = ATTRIBUTE_LABELS[attribute.key];
</script>

<button
  type="button"
  class="rounded-md border border-solid px-2.5 py-1 transition-all focus-visible:ring-2 focus-visible:ring-accent-accent focus-visible:ring-offset-2 focus-visible:ring-offset-bg-secondary"
  class:border-border-primary={!attribute.enabled}
  class:bg-transparent={!attribute.enabled}
  class:text-text-secondary={!attribute.enabled}
  class:hover:bg-bg-button-hover={!attribute.enabled}
  class:hover:text-text-primary={!attribute.enabled}
  class:border-accent-accent={attribute.enabled}
  class:bg-accent-accent={attribute.enabled}
  class:text-text-white={attribute.enabled}
  class:hover:bg-accent-primary-hover={attribute.enabled}
  aria-pressed={attribute.enabled}
  title={label}
  on:click
>
  <div class="text-[12px] whitespace-nowrap font-medium leading-4">
    {label}
  </div>
</button>
