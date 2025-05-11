<script lang="ts">
  import ATab from "@/components/Tab/ATab.svelte";
  import ScrollableHorizontal from "@/components/UI/ScrollableHorizontal.svelte";
  import { selected, tabs, reorderTabs, type Tab as TabType, getSelectedTab } from "@/store/tabs";
  import { onDestroy, onMount, tick } from "svelte";
  import { push } from "svelte-spa-router";

  let tabElements: HTMLElement[] = [];
  let draggingTabId: number | null = null;
  let draggingTabIndex: number | null = null;
  let originalTabRect: DOMRect | null = null;
  let placeholderIndex: number | null = null;
  let isActuallyDragging = false;

  let showGhostTab = false;
  let ghostTabContent: TabType | null = null;
  let ghostTabLeft = 0;
  let ghostTabTop = 0;
  let ghostTabWidth = 0;
  let ghostTabHeight = 0;

  const DRAG_START_THRESHOLD = 5;

  // --- 追加: リスト全体でドラッグ操作中かどうかのフラグ ---
  $: isAnyTabDragging = draggingTabId !== null && isActuallyDragging;
  // --- ここまで追加 ---

  const handleMouseDown = (event: MouseEvent & { currentTarget: EventTarget & HTMLDivElement }, tabData: TabType, index: number) => {
    isActuallyDragging = false;

    const currentSelectedTab = getSelectedTab();
    if (!currentSelectedTab || currentSelectedTab.id !== tabData.id) {
      push(`/${tabData.type}/${tabData.workId}`);
    }

    draggingTabId = tabData.id;
    draggingTabIndex = index;

    const tabElement = tabElements[index];
    if (!tabElement) return;

    originalTabRect = tabElement.getBoundingClientRect();

    ghostTabContent = { ...tabData };
    ghostTabWidth = originalTabRect.width;
    ghostTabHeight = originalTabRect.height;

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  const handleMouseMove = async (event: MouseEvent) => {
    if (draggingTabId === null || draggingTabIndex === null || !originalTabRect || !ghostTabContent) return;
    event.preventDefault();

    if (!isActuallyDragging) {
      const currentSelectedTab = getSelectedTab();
      if (currentSelectedTab && currentSelectedTab.id === draggingTabId) {
        const deltaXSinceMouseDown = event.clientX - (originalTabRect.left);
        const deltaYSinceMouseDown = event.clientY - (originalTabRect.top);
        if (Math.sqrt(deltaXSinceMouseDown**2 + deltaYSinceMouseDown**2) > DRAG_START_THRESHOLD) {
          isActuallyDragging = true;
          showGhostTab = true;
        } else {
          return;
        }
      } else {
        return;
      }
    }

    ghostTabLeft = event.clientX;
    ghostTabTop = event.clientY;

    const currentTabs = $tabs;
    if (currentTabs.length === 0 && draggingTabIndex !== 0) { // 最後のタブをドラッグして0個になった場合など
        placeholderIndex = 0; // 唯一のドロップ先としてリストの先頭
        await tick();
        return;
    }
    if (currentTabs.length === 0 && draggingTabIndex === 0){ // タブが元々0個か、1個のタブをドラッグ中の場合
        placeholderIndex = 0;
        await tick();
        return;
    }


    let newPlaceholderIndex = draggingTabIndex; // 初期値は元の位置（実質インジケータなし）

    // targetElementsForIndicator は、ドラッグ中の元のタブ（プレースホルダー）を除いた、
    // インジケータの位置を計算するための基準となるタブ要素のリストと考える。
    // しかし、実際には元のタブもレイアウト上は存在するので、全てのタブを対象にする。
    for (let i = 0; i < currentTabs.length; i++) {
      const otherTabElement = tabElements[i];
      if (!otherTabElement) continue;

      // ドラッグ中のタブ（プレースホルダー）自身を基準にインジケータを出すことはないので、
      // このタブの左か右かを判定する。
      const rect = otherTabElement.getBoundingClientRect();
      const otherTabMidX = rect.left + rect.width / 2;

      if (event.clientX < otherTabMidX) {
        // マウスが i 番目のタブの中央より左 => i 番目のタブの「前」にインジケータ
        newPlaceholderIndex = i;
        break;
      }
      // マウスが i 番目のタブの中央より右 => i 番目のタブの「後」にインジケータの可能性
      newPlaceholderIndex = i + 1;
    }

    newPlaceholderIndex = Math.max(0, Math.min(currentTabs.length, newPlaceholderIndex));

    if (placeholderIndex !== newPlaceholderIndex) {
        placeholderIndex = newPlaceholderIndex;
        await tick();
    }
  };

  const resetDragState = () => {
    draggingTabId = null;
    draggingTabIndex = null;
    originalTabRect = null;
    placeholderIndex = null;
    isActuallyDragging = false;
    showGhostTab = false;
    ghostTabContent = null;
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  }

  const handleMouseUp = () => {
    if (draggingTabId === null || draggingTabIndex === null) {
        resetDragState();
        return;
    }

    if (isActuallyDragging && placeholderIndex !== null) {
        let finalNewIndex = placeholderIndex;
        if (draggingTabIndex < finalNewIndex) {
            finalNewIndex--;
        }
        if (draggingTabIndex !== finalNewIndex) {
            reorderTabs(draggingTabIndex, finalNewIndex);
        }
    }
    resetDragState();
  };

  onMount(() => {
    return () => {
      resetDragState();
    };
  });

</script>

<ScrollableHorizontal>
  <div class="grid-(~ cols-[min-content_1fr]) items-center">
    <div class="flex items-center h-10 relative" style="min-height: 2.5rem;">
      {#each $tabs as tab, i (tab.id)}
        <div
          bind:this={tabElements[i]}
          class="tab-wrapper"
          class:is-drag-placeholder={draggingTabId === tab.id && isActuallyDragging}
          style="order: {i};"
          on:mousedown={(e) => handleMouseDown(e, tab, i)}
        >
          <ATab {tab} selected={$selected === i}
                isDragging={false}
                isPlaceholder={draggingTabId === tab.id && isActuallyDragging}
                isAnyTabDragging={isAnyTabDragging}
          />
        </div>
      {/each}

      {#if isActuallyDragging && placeholderIndex !== null}
        {#if placeholderIndex === $tabs.length}
          {@const lastTabElement = $tabs.length > 0 ? tabElements[$tabs.length - 1] : null}
          {#if lastTabElement}
            <div class="drop-indicator" style="left: {lastTabElement.offsetLeft + lastTabElement.offsetWidth}px;"></div>
          {:else if $tabs.length === 0 || ($tabs.length === 1 && draggingTabIndex === 0) }
            <div class="drop-indicator" style="left: 0px;"></div>
          {/if}
        {:else if tabElements[placeholderIndex]}
          {@const targetElement = tabElements[placeholderIndex]}
          <div class="drop-indicator" style="left: {targetElement.offsetLeft}px;"></div>
        {/if}
      {/if}
    </div>
    <div
      class="w-full h-full bg-bg-disabled border-(b-1px solid border-primary)"
    />
  </div>
</ScrollableHorizontal>

{#if showGhostTab && ghostTabContent}
  <div
    class="ghost-tab"
    style="
      position: fixed;
      left: {ghostTabLeft}px;
      top: {ghostTabTop}px;
      width: {ghostTabWidth}px;
      height: {ghostTabHeight}px;
      z-index: 1000;
      pointer-events: none;
      transform: scale(1.02);
    "
  >
    <!-- ゴーストタブ自身には isAnyTabDragging は不要（常に isDragging=true で制御） -->
    <ATab tab={ghostTabContent} selected={false} isDragging={true} />
  </div>
{/if}

<style>
  .tab-wrapper {
    position: relative;
  }
  .ghost-tab {
    opacity: 0.7;
    overflow: hidden;
    background-color: #2d333b;
  }
  .drop-indicator {
    position: absolute;
    top: 0px;
    bottom: 0px;
    width: 2px;
    background-color: #d8d8d8;
    z-index: 20;
    pointer-events: none;
  }
</style>
