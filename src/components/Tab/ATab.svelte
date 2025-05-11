<script lang="ts">
  import { deleteTab, type Tab } from "@/store/tabs";
  import { push } from "svelte-spa-router";

  export let tab: Tab;
  export let selected: boolean;          // このタブが現在選択されているか
  export let isDragging: boolean = false;    // このATabインスタンスが「ゴーストタブ」として機能しているか
  export let isPlaceholder: boolean = false; // このATabインスタンスが「プレースホルダー」として機能しているか
  export let isAnyTabDragging: boolean = false; // いずれかのタブがドラッグされているか (リスト全体の状態)

  $: tabIcon =
    tab.type === "works"
      ? "i-material-symbols-computer-outline-rounded color-accent-accent"
      : tab.type === "memos"
      ? "i-material-symbols-drive-file-rename-outline color-accent-edit"
      : "";

  const closeWheelClick = (e: MouseEvent) => {
    if (isDragging || isPlaceholder || isAnyTabDragging) return;
    if (e.button === 1) {
      deleteTab(tab.id);
    }
  };

  const handleClick = () => {
    if (isDragging || isPlaceholder || isAnyTabDragging) return;
    push(`/${tab.type}/${tab.workId}`);
  };

  // スタイルを決定するためのリアクティブな変数
  $: isActiveLook = selected && !isPlaceholder && !isDragging; // 通常のアクティブ状態
  $: isGhostOrPlaceholderActiveLook = isDragging || isPlaceholder; // ゴーストまたはプレースホルダーの時に注目風にするか

</script>

<div
  on:click={handleClick}
  on:mousedown={closeWheelClick}
  class="tab-container"
  class:cursor-pointer={!isAnyTabDragging && !isPlaceholder && !isDragging}
  class:cursor-grabbing={isDragging && !isPlaceholder}
  class:placeholder-style={isPlaceholder}
>
  <div
    class="tab-content-area flex items-center gap-2 px-3 h-10 transition-all border-(b-1px r-1px solid border-primary) group max-w-60"
    class:bg-bg-primary={isActiveLook || isGhostOrPlaceholderActiveLook}
    class:border-b-transparent={isActiveLook || isGhostOrPlaceholderActiveLook}
    class:bg-bg-disabled={!isActiveLook && !isGhostOrPlaceholderActiveLook}
    class:hover:bg-bg-primary={!selected && !isPlaceholder && !isDragging && !isAnyTabDragging}
    class:ghost-appearance={isDragging}
  >
    <div class="{tabIcon} w-5 h-5 flex-shrink-0" />
    <div
      class="tab-title text-body2 whitespace-nowrap text-ellipsis overflow-hidden"
      class:text-text-primary={isActiveLook || isGhostOrPlaceholderActiveLook}
      class:text-text-tertiary={!isActiveLook && !isGhostOrPlaceholderActiveLook}
    >
      {tab.title}
    </div>
    <div
      class="rounded flex items-center justify-center transition-all"
      class:hover:bg-bg-secondary={!isAnyTabDragging && !isPlaceholder}
      style="opacity: {isPlaceholder || isDragging || isAnyTabDragging ? 0 : ''};"
    >
      <button
        class="group-hover:opacity-100 opacity-0 transition-all w-5 h-5 i-iconoir-cancel"
        class:color-text-secondary={isActiveLook || isGhostOrPlaceholderActiveLook}
        class:color-text-tertiary={!isActiveLook && !isGhostOrPlaceholderActiveLook}
        on:click|stopPropagation={() => {
            if (isDragging || isPlaceholder || isAnyTabDragging) return;
            deleteTab(tab.id);
        }}
        tabindex={isPlaceholder || isDragging || isAnyTabDragging ? -1 : 0}
      />
    </div>
  </div>
</div>

<style>
  .tab-container {
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
  }
  .placeholder-style {
    /* ATabList側の .is-drag-placeholder で opacity を制御 */
  }
  .ghost-appearance {
    /* ゴーストタブがATabコンポーネントで描画される際の追加スタイル */
    /* 例: ATabListの.ghost-tabのスタイルと合わせる */
  }
</style>
