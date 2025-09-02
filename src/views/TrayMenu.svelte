<script lang="ts">
  import { appWindow } from "@tauri-apps/api/window";
  import { exit } from "@tauri-apps/api/process";
  import { emit, listen } from "@tauri-apps/api/event";

  async function showMainWindow() {
    await emit("show_main_window");
  }

  async function quitApp() {
    await exit(0);
  }

  listen("blur", () => {
    appWindow.hide();
  });
</script>

<div
  class="flex flex-col gap-2 p-4 bg-gray-800/70 text-white rounded-lg backdrop-blur-sm"
  on:mouseleave={() => appWindow.hide()}
>
  <button on:click={showMainWindow} class="p-2 hover:bg-gray-700 rounded text-left">
    Show Main Window
  </button>
  <button on:click={quitApp} class="p-2 hover:bg-gray-700 rounded text-left">
    Quit
  </button>
</div>

<style>
  :global(body) {
    background-color: transparent;
  }
</style>
