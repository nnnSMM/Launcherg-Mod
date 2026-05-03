<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const appWindow = getCurrentWindow();
  let isMaximized = false;

  onMount(async () => {
    isMaximized = await appWindow.isMaximized();

    const unlisten = await appWindow.onResized(async () => {
      isMaximized = await appWindow.isMaximized();
    });

    return () => {
      unlisten();
    };
  });

  function minimize() {
    console.log("minimize clicked");
    appWindow.minimize().catch(console.error);
  }

  function toggleMaximize() {
    console.log("toggleMaximize clicked");
    appWindow.toggleMaximize().catch(console.error);
  }

  function closeWindow() {
    console.log("close clicked");
    appWindow.close().catch(console.error);
  }

  function startDragging(e: MouseEvent) {
    if (e.buttons === 1) {
      appWindow.startDragging().catch(console.error);
    }
  }
</script>

<div class="h-8 bg-bg-secondary flex justify-between items-center select-none w-full z-50 shrink-0">
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div on:mousedown={startDragging} class="flex items-center pl-3 h-full flex-1 cursor-default">
    <!-- Icon or Title can go here -->
    <span class="text-text-secondary text-sm font-bold pointer-events-none">Launcherg</span>
  </div>
  <div class="flex h-full">
    <button tabindex="-1" on:click={minimize} class="w-11 h-full flex items-center justify-center bg-transparent hover:bg-bg-tertiary transition-colors text-text-secondary hover:text-text-primary outline-none border-none focus:outline-none focus-visible:outline-none">
      <div class="i-material-symbols:remove text-lg"></div>
    </button>
    <button tabindex="-1" on:click={toggleMaximize} class="w-11 h-full flex items-center justify-center bg-transparent hover:bg-bg-tertiary transition-colors text-text-secondary hover:text-text-primary outline-none border-none focus:outline-none focus-visible:outline-none">
      <div class={isMaximized ? "i-material-symbols:filter-none text-[14px]" : "i-material-symbols:check-box-outline-blank text-base"}></div>
    </button>
    <button tabindex="-1" on:click={closeWindow} class="w-11 h-full flex items-center justify-center bg-transparent hover:bg-accent-error transition-colors text-text-secondary hover:text-text-white outline-none border-none focus:outline-none focus-visible:outline-none">
      <div class="i-material-symbols:close text-lg"></div>
    </button>
  </div>
</div>
