<script lang="ts">
  import type { CollectionElement, PlayStatus as PlayStatusType } from "@/lib/types";
  import type { Readable, Writable } from "svelte/store";
  import { onDestroy, onMount } from "svelte";
  import GameListItemSelectable from "@/components/PlayStatusBulkEditor/GameListItemSelectable.svelte";

  export let elementsStore: Readable<CollectionElement[]>;
  export let selectedIdsStore: Writable<Set<number>>;
  export let onToggleSelection: (id: number) => void;
  export let previewTargetPlayStatus: PlayStatusType;

  export let itemHeight: number = 72;

  let scrollContainerElement: HTMLElement;
  let scrollTop: number = 0;
  let viewportHeight: number = 0;

  let visibleItems: CollectionElement[] = [];
  let contentOffsetY: number = 0;
  let totalContentHeight: number = 0;

  const bufferItems = 3;

  function updateVisibleRange() {
    if (!$elementsStore || $elementsStore.length === 0 || !scrollContainerElement) {
      visibleItems = [];
      contentOffsetY = 0;
      totalContentHeight = 0;
      return;
    }

    totalContentHeight = $elementsStore.length * itemHeight;
    viewportHeight = scrollContainerElement.clientHeight;

    const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - bufferItems);
    const endIndex = Math.min(
      $elementsStore.length - 1,
      Math.ceil((scrollTop + viewportHeight) / itemHeight) - 1 + bufferItems
    );

    visibleItems = $elementsStore.slice(startIndex, endIndex + 1);
    contentOffsetY = startIndex * itemHeight;
  }

  const handleScroll = (event: Event) => {
    scrollTop = (event.target as HTMLElement).scrollTop;
    updateVisibleRange();
  };

  $: if ($elementsStore && scrollContainerElement) {
    updateVisibleRange();
  }

  let resizeObserver: ResizeObserver;
  onMount(() => {
    if (scrollContainerElement) {
      viewportHeight = scrollContainerElement.clientHeight;
      updateVisibleRange();

      resizeObserver = new ResizeObserver(() => {
        if (scrollContainerElement) {
          viewportHeight = scrollContainerElement.clientHeight;
          updateVisibleRange();
        }
      });
      resizeObserver.observe(scrollContainerElement);
    }
    return () => {
      if (resizeObserver && scrollContainerElement) {
        resizeObserver.unobserve(scrollContainerElement);
      }
    };
  });

</script>

<div
  bind:this={scrollContainerElement}
  class="list-scroll-container w-full h-full overflow-y-auto p-2 md:p-4"
  on:scroll={handleScroll}
>
  <div
    class="list-content-wrapper relative"
    style="height: {totalContentHeight}px;"
  >
    {#each visibleItems as game (game.id)}
      {@const itemIndexInFullList = $elementsStore.findIndex(el => el.id === game.id)}
      {#if itemIndexInFullList !== -1}
        <div
          class="list-item-wrapper w-full absolute"
          style="top: {itemIndexInFullList * itemHeight}px; left: 0; right: 0; height: {itemHeight}px;"
        >
          <GameListItemSelectable
            {game}
            isSelected={$selectedIdsStore.has(game.id)}
            previewTargetPlayStatus={$selectedIdsStore.has(game.id) ? previewTargetPlayStatus : undefined}
            on:toggle={() => onToggleSelection(game.id)}
          />
        </div>
      {/if}
    {/each}
  </div>
</div>
