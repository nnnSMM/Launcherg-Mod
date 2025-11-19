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
  // export let attributes: Attribute[]; // 変更: このプロパティは削除
  export let playStatusAttributes: Attribute[]; // 追加: プレイ状況属性リスト
  export let otherAttributes: Attribute[]; // 追加: その他の属性リスト

  const dispatcher = createEventDispatcher<{
    toggleAttributeEnabled: { key: AttributeKey };
  }>();

  // otherAttributes 用のスクロール制御
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
</script>

<div class="space-y-2 w-full">
  <div class="flex items-center gap-2">
    <div class="flex-1">
      <SearchInput
        bind:value={query}
        placeholder="Filter by title, brand and more"
      />
    </div>
    <APopover panelClass="right-0" let:close>
      <ButtonBase
        appendClass="h-8 w-8 flex items-center justify-center"
        tooltip={{
          content: "ゲームの並べ替え",
          placement: "bottom",
          theme: "default",
          delay: 1000,
        }}
        slot="button"
      >
        <div
          class="color-ui-tertiary w-5 h-5 i-material-symbols-sort-rounded"
        />
      </ButtonBase>
      <SortPopover bind:value={order} on:close={() => close(null)} />
    </APopover>
  </div>

  <!-- プレイ状況ボタンセクション -->
  {#if playStatusAttributes && playStatusAttributes.length > 0}
    <div class="flex flex-wrap items-center gap-2 pb-1">
      {#each playStatusAttributes as attribute (attribute.key)}
        <SearchAttribute
          {attribute}
          on:click={() =>
            dispatcher("toggleAttributeEnabled", { key: attribute.key })}
        />
      {/each}
    </div>
  {/if}

  <!-- その他の属性ボタンセクション -->
  {#if otherAttributes && otherAttributes.length > 0}
    <div class="relative">
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
