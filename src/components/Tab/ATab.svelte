<script lang="ts">
  import { deleteTab, type Tab } from "@/store/tabs";
  import { push } from "svelte-spa-router";

  export let tab: Tab;
  export let selected: boolean;
  export let isDragging: boolean = false;
  export let isPlaceholder: boolean = false;
  export let isAnyTabDragging: boolean = false;

  $: tabIcon =
    tab.type === "works"
      ? "i-material-symbols-computer-outline-rounded color-accent-accent"
      : tab.type === "memos"
      ? "i-material-symbols-drive-file-rename-outline color-accent-edit"
      : tab.type === "settings"
      ? "i-material-symbols-label-outline-rounded color-text-tertiary"
      : "";

  const closeWheelClick = (e: MouseEvent) => {
    if (isDragging || isPlaceholder || isAnyTabDragging) return;
    if (e.button === 1) {
      deleteTab(tab.id);
    }
  };

  const handleClick = () => {
    if (isDragging || isPlaceholder || isAnyTabDragging) return;
    push(tab.path);
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      handleClick();
    }
  };
</script>

<div
  role="button"
  tabindex="0"
  on:click={handleClick}
  on:keydown={handleKeyDown}
  on:mousedown={closeWheelClick}
  class="tab-container"
  class:cursor-pointer={!isAnyTabDragging && !isPlaceholder && !isDragging}
  class:cursor-grabbing={isDragging && !isPlaceholder}
  class:placeholder-style={isPlaceholder}
>
  <div
    class="tab-content-area flex items-center gap-2 px-3 h-10 transition-all border-(b-1px r-1px solid border-primary) group max-w-60"
    class:bg-bg-primary={(selected && !isPlaceholder && !isDragging)}
    class:border-b-transparent={(selected && !isPlaceholder && !isDragging)}
    class:bg-bg-disabled={(!selected && !isPlaceholder && !isDragging)}
    class:hover:bg-bg-primary={(!selected && !isPlaceholder && !isDragging && !isAnyTabDragging)}
    class:ghost-appearance={isDragging}
  >
    <div class="{tabIcon} w-5 h-5 flex-shrink-0" />
    <div
      class="tab-title text-body2 whitespace-nowrap text-ellipsis overflow-hidden"
      class:text-text-primary={(selected && !isPlaceholder && !isDragging)}
      class:text-text-tertiary={((!selected || isPlaceholder || isDragging))}
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
        class:color-text-secondary={(selected && !isPlaceholder && !isDragging)}
        class:color-text-tertiary={((!selected || isPlaceholder || isDragging))}
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
  }
  .ghost-appearance {
  }
</style>
