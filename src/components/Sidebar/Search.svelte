<script lang="ts">
  import SearchAttribute from "@/components/Sidebar/SearchAttribute.svelte";
  import SearchInput from "@/components/Sidebar/SearchInput.svelte";
  import SortPopover from "@/components/Sidebar/SortPopover.svelte";
  import {
    type Attribute,
    type AttributeKey,
  } from "@/components/Sidebar/searchAttributes";
  import type { SortOrder } from "@/components/Sidebar/sort";
  import APopover from "@/components/UI/APopover.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import ScrollableHorizontal from "@/components/UI/ScrollableHorizontal.svelte";
  import { createEventDispatcher, type SvelteComponent } from "svelte";

  export let query: string;
  export let order: SortOrder;
  export let playStatusAttributes: Attribute[];
  export let otherAttributes: Attribute[];

  const dispatcher = createEventDispatcher<{
    toggleAttributeEnabled: { key: AttributeKey };
  }>();

  const labels = {
    search: "\u30bf\u30a4\u30c8\u30eb\u30fb\u30d6\u30e9\u30f3\u30c9\u3067\u691c\u7d22",
    sort: "\u30b2\u30fc\u30e0\u306e\u4e26\u3079\u66ff\u3048",
    status: "\u30b9\u30c6\u30fc\u30bf\u30b9",
    filters: "\u30d5\u30a3\u30eb\u30bf",
  };

  let isShowBackOther = false;
  let isShowForwardOther = true;
  const onScrollOther = (e: Event) => {
    const element = e.target as HTMLElement;
    const rect = element.getBoundingClientRect();
    const width = element.scrollWidth;

    const left = element.scrollLeft;
    const right = width - rect.width - left;

    isShowBackOther = left > 0;
    isShowForwardOther = right > 0;
  };
  let scrollableOther: SvelteComponent;

  let isShowBackPlay = false;
  let isShowForwardPlay = true;
  const onScrollPlay = (e: Event) => {
    const element = e.target as HTMLElement;
    const rect = element.getBoundingClientRect();
    const width = element.scrollWidth;

    const left = element.scrollLeft;
    const right = width - rect.width - left;

    isShowBackPlay = left > 0;
    isShowForwardPlay = right > 0;
  };
  let scrollablePlay: SvelteComponent;
</script>

<div class="space-y-2 w-full min-w-0">
  <div class="flex items-center gap-2 min-w-0">
    <div class="flex-1 min-w-0">
      <SearchInput
        bind:value={query}
        placeholder={labels.search}
        ariaLabel={labels.search}
      />
    </div>
    <APopover panelClass="right-0" let:close>
      <ButtonBase
        appendClass="h-8 w-8 flex items-center justify-center !bg-transparent !border-border-primary !border-opacity-100 hover:!border-border-button-hover hover:!bg-white/10"
        tooltip={{
          content: labels.sort,
          placement: "bottom",
          theme: "default",
          delay: 1000,
        }}
        slot="button"
        ariaLabel={labels.sort}
      >
        <div
          class="color-ui-tertiary w-5 h-5 i-material-symbols-sort-rounded"
        />
      </ButtonBase>
      <SortPopover bind:value={order} on:close={() => close(null)} />
    </APopover>
  </div>

  {#if playStatusAttributes && playStatusAttributes.length > 0}
    <div class="relative min-w-0 rounded-lg border border-border-primary bg-bg-primary/20 px-2 py-2">
      <div class="mb-1 px-1 text-[11px] font-semibold uppercase tracking-wide text-text-tertiary">
        {labels.status}
      </div>
      <div
        class="hide-scrollbar"
        style="mask-image: linear-gradient(to right, {isShowBackPlay
          ? 'transparent, black 120px'
          : 'black 0px'}, {isShowForwardPlay
          ? 'black calc(100% - 120px), transparent'
          : 'black 100%'}); -webkit-mask-image: linear-gradient(to right, {isShowBackPlay
          ? 'transparent, black 120px'
          : 'black 0px'}, {isShowForwardPlay
          ? 'black calc(100% - 120px), transparent'
          : 'black 100%'});"
      >
        <ScrollableHorizontal
          on:scroll={(e) => onScrollPlay(e.detail.event)}
          bind:this={scrollablePlay}
        >
          <div class="flex items-center gap-2 pb-1">
            {#each playStatusAttributes as attribute (attribute.key)}
              <SearchAttribute
                {attribute}
                on:click={() =>
                  dispatcher("toggleAttributeEnabled", { key: attribute.key })}
              />
            {/each}
          </div>
        </ScrollableHorizontal>
      </div>
    </div>
  {/if}

  {#if otherAttributes && otherAttributes.length > 0}
    <div class="relative rounded-lg border border-border-primary bg-bg-primary/20 px-2 py-2">
      <div class="mb-1 px-1 text-[11px] font-semibold uppercase tracking-wide text-text-tertiary">
        {labels.filters}
      </div>
      <div
        class="hide-scrollbar"
        style="mask-image: linear-gradient(to right, {isShowBackOther
          ? 'transparent, black 120px'
          : 'black 0px'}, {isShowForwardOther
          ? 'black calc(100% - 120px), transparent'
          : 'black 100%'}); -webkit-mask-image: linear-gradient(to right, {isShowBackOther
          ? 'transparent, black 120px'
          : 'black 0px'}, {isShowForwardOther
          ? 'black calc(100% - 120px), transparent'
          : 'black 100%'});"
      >
        <ScrollableHorizontal
          on:scroll={(e) => onScrollOther(e.detail.event)}
          bind:this={scrollableOther}
        >
          <div class="flex items-center gap-2 pb-1">
            {#each otherAttributes as attribute (attribute.key)}
              <SearchAttribute
                {attribute}
                on:click={() =>
                  dispatcher("toggleAttributeEnabled", { key: attribute.key })}
              />
            {/each}
          </div>
        </ScrollableHorizontal>
      </div>
    </div>
  {/if}
</div>
