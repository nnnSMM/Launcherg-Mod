<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import { useFilter } from "@/lib/filter";
  import { type Option } from "@/lib/trieFilter";
  import { createWritable } from "@/lib/utils";
  import { createEventDispatcher } from "svelte";
  import { writable } from "svelte/store";

  export let options: Option<string | number>[];
  export let title: string | undefined = undefined;
  export let enableFilter: boolean = false;
  export let showSelectedCheck = true;
  export let showSelectedBackground = true;
  export let filterPlaceholder = "";
  export let bottomCreateButtonText = "";
  export let value: string | number;

  const [writableOptions, getOptions] = createWritable(options);
  $: {
    writableOptions.set(options);
  }

  const localQuery = writable("");
  const { filtered } = useFilter(localQuery, writableOptions, getOptions);

  const dispatcher = createEventDispatcher<{
    select: { value: string | number };
    create: {};
    close: {};
  }>();
</script>

<div class="max-h-80 overflow-hidden flex flex-col">
  {#if title}
    <div class="flex items-center gap-8 border-b-1px border-border-primary border-solid">
      <div
        class="whitespace-nowrap p-x-4 p-y-2 text-body2 text-text-primary font-bold"
      >
        {title}
      </div>
    </div>
  {/if}
  {#if enableFilter}
    <div class="p-2 border-b-1px border-border-primary border-solid">
      <Input
        bind:value={$localQuery}
        placeholder={filterPlaceholder}
        bgClass="bg-bg-primary/20"
        autofocus
      />
    </div>
  {/if}
  <div class="flex flex-col overflow-y-auto min-h-full">
    {#each $filtered as option, i (option)}
      <button
        class={`${showSelectedBackground && value === option.value ? "bg-bg-tertiary/40" : "bg-transparent"}
                p-x-4 p-y-1 ${
                  options.length - 1 !== i
                    ? "border-b-1px border-solid border-border-primary"
                    : ""
                } hover:bg-accent-primary/16 w-full flex items-center gap-2 transition-all cursor-pointer`}
        on:click={() => {
          value = option.value;
          dispatcher("select", { value: option.value });
          $localQuery = "";
          dispatcher("close");
        }}
      >
        {#if showSelectedCheck}
          <div
            class="h-5 w-5 color-text-primary"
            class:i-material-symbols-check-small-rounded={value ===
              option.value}
          />
        {/if}
        <div class="text-body2 text-text-primary font-medium">
          {option.label}
        </div>
      </button>
    {/each}
  </div>
  {#if bottomCreateButtonText}
    <button
      class="bg-transparent hover:bg-accent-primary/16 transition-all w-full p-l-4 p-r-5 p-y-2 flex items-center border-t-1px border-solid border-border-primary"
      on:click={() => dispatcher("create")}
    >
      <div class="w-5 h-5 i-iconoir-plus color-text-primary" />
      <div class="text-text-primary text-body2 text-left font-bold whitespace-nowrap">
        {bottomCreateButtonText}
      </div>
    </button>
  {/if}
</div>
