<script lang="ts">
  import GameTileSelectable from "@/components/UI/GameTileSelectable.svelte";
  import { usePlayStatusVirtualScrollerMasonry } from "@/components/PlayStatusBulkEditor/usePlayStatusVirtualScrollerMasonry";
  import type { CollectionElement, PlayStatus as PlayStatusType } from "@/lib/types";
  import type { Readable, Writable } from "svelte/store";

  export let elementsStore: Readable<CollectionElement[]>;
  export let selectedIdsStore: Writable<Set<number>>;
  export let onToggleSelection: (id: number) => void;
  export let previewTargetPlayStatus: PlayStatusType;

  export let setVirtualHeight: (v: number) => void;
  export let contentsWidth: Readable<number>;
  export let contentsScrollY: Readable<number>;
  export let containerHeight: Readable<number>;
  export let contentsScrollTo: (v: number) => void;

  export let minItemWidth: number = 16 * 10;    // タイル全体の目標最小幅
  export let itemGap: number = 12;
  // fixedThumbnailWidth は削除
  export let titleAreaHeight: number = 40;
  export let placeholderAspectRatio: number = 4 / 3;
  export let tileInternalPadding: number = 8;

  const { visibleLayouts } = usePlayStatusVirtualScrollerMasonry(
    elementsStore,
    setVirtualHeight,
    contentsWidth,
    contentsScrollY,
    containerHeight,
    {
      minItemWidth,
      itemGap,
      tileInternalPadding, // ★フックに渡す
      titleAreaHeight,
      placeholderAspectRatio
    }
  );
</script>

<div>
  {#each $visibleLayouts as { top, left, width, height, element, imgDisplayWidth, imgDisplayHeight } (element.id)}
    <div
      class="absolute"
      style="left: {left}px; top: {top}px; width: {width}px; height: {height}px; display: flex; justify-content: center;"
    >
      <GameTileSelectable
        game={element}
        isSelected={$selectedIdsStore.has(element.id)}
        previewTargetPlayStatus={$selectedIdsStore.has(element.id) ? previewTargetPlayStatus : undefined}
        on:toggle={() => onToggleSelection(element.id)}
        columnWidth={width}
        itemHeight={height}
        targetImageWidth={imgDisplayWidth}
        imageDisplayHeight={imgDisplayHeight}
        tilePadding={tileInternalPadding}
      />
    </div>
  {/each}
</div>
