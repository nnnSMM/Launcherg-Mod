<script lang="ts">
  import SelectOptions from "@/components/UI/SelectOptions.svelte";
  import APopover from "@/components/UI/APopover.svelte";
  import { createEventDispatcher } from "svelte";
  import { type Option } from "@/lib/trieFilter";

  export let options: Option<string | number>[];
  export let value: Option<string | number>["value"];
  export let iconClass: string = "";
  export let title: string | undefined = undefined;
  export let enableFilter: boolean = false;
  export let filterPlaceholder = "";
  export let bottomCreateButtonText = "";
  export let showSelectedCheck = false;
  export let showSelectedBackground = true;
  export let buttonBorderless = false;
  export let popoverPlacement: "auto" | "top" | "bottom" = "auto";

  $: selectedLabel = options.find((v) => v.value === value)?.label ?? "";
  $: buttonBorderlessClass = buttonBorderless
    ? "!border-transparent !border-opacity-0 hover:!border-transparent hover:!border-opacity-0"
    : "";

  const dispather = createEventDispatcher<{ create: {} }>();
</script>

<APopover let:open let:close placement={popoverPlacement}>
  <div slot="button">
    <slot>
      <button
        class={`h-8 w-full flex items-center gap-2 border border-border-button border-opacity-10 border-solid rounded bg-bg-button px-3 transition-all hover:border-border-button-hover hover:bg-bg-button-hover overflow-hidden ${buttonBorderlessClass}`}
      >
        {#if iconClass}
          <div class={`${iconClass} w-4 h-4`} />
        {/if}
        <div class="text-body text-text-primary font-bold max-h-full">
          {selectedLabel}
        </div>
        <div
          class="i-material-symbols-arrow-drop-down ml-auto h-4 w-4 color-text-primary transition-all flex-shrink-0"
          class:rotate-180={open}
        />
      </button>
    </slot>
  </div>
  <SelectOptions
    {title}
    {enableFilter}
    {filterPlaceholder}
    {options}
    {bottomCreateButtonText}
    {showSelectedCheck}
    {showSelectedBackground}
    bind:value
    on:select
    on:create={() => {
      close(null);
      dispather("create");
    }}
    on:close={() => close(null)}
  />
</APopover>
