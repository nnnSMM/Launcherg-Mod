<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "/icon.png";
  import { link, push } from "svelte-spa-router";
  import {
    openDisplaySettingsTab,
    openSettingsTab,
    openShortcutSettingsTab,
  } from "@/store/tabs";
  import APopover from "@/components/UI/APopover.svelte";
  import ImportPopover from "@/components/Sidebar/ImportPopover.svelte";
  import ImportAutomatically from "@/components/Sidebar/ImportAutomatically.svelte";
  import ImportManually from "@/components/Sidebar/ImportManually.svelte";
  import {
    commandSaveMainWindowState,
    commandUpsertCollectionElement,
  } from "@/lib/command";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { showErrorToast, showInfoToast } from "@/lib/toast";
  import type { AllGameCacheOne } from "@/lib/types";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { enqueueGameScreenshotPrefetch } from "@/lib/useGameScreenshots";

  export let variant: "main" | "screenshot" = "main";
  export let heightClass: string = "h-8";

  const appWindow = getCurrentWindow();
  const isDemoBuild = import.meta.env.BASE_URL === "./";
  const navLabels = {
    home: "\u30db\u30fc\u30e0",
    gameAdd: "\u30b2\u30fc\u30e0\u8ffd\u52a0",
    shortcut: "\u30b7\u30e7\u30fc\u30c8\u30ab\u30c3\u30c8",
    display: "\u8868\u793a",
    bulkEdit: "\u4e00\u62ec\u7de8\u96c6",
    help: "\u30d8\u30eb\u30d7",
  };
  const demoRegistrationDisabledMessage =
    "demo \u3067\u306f\u30b2\u30fc\u30e0\u767b\u9332\u306f\u3067\u304d\u307e\u305b\u3093\u3002\u30db\u30fc\u30e0\u306e\u300c\u30d5\u30a9\u30eb\u30c0\u7d10\u3065\u3051\u3092\u8a66\u3059\u300d\u3067\u5224\u5b9a\u3060\u3051\u78ba\u8a8d\u3067\u304d\u307e\u3059\u3002";

  let isMaximized = false;
  let isOpenImportAutomatically = false;
  let isOpenImportManually = false;

  const importManually = async (arg: {
    exePath: string | null;
    lnkPath: string | null;
    gameCache: AllGameCacheOne;
  }) => {
    if (isDemoBuild) {
      showInfoToast(demoRegistrationDisabledMessage);
      isOpenImportManually = false;
      return;
    }
    try {
      await commandUpsertCollectionElement(arg);
      try {
        await registerCollectionElementDetails();
      } catch (e) {
        console.error("Failed to fetch extended game details:", e);
      }
      showInfoToast(`${arg.gameCache.gamename}\u3092\u767b\u9332\u3057\u307e\u3057\u305f\u3002`);
    } catch (e) {
      console.error("Failed to add game to collection:", e);
      showErrorToast(`${arg.gameCache.gamename}\u306e\u767b\u9332\u306b\u5931\u6557\u3057\u307e\u3057\u305f\u3002`);
    } finally {
      await sidebarCollectionElements.refetch();
      const imported = sidebarCollectionElements
        .value()
        .find((v) => v.id === arg.gameCache.id);
      if (imported) {
        enqueueGameScreenshotPrefetch([imported]);
      }
      isOpenImportManually = false;
    }
  };

  onMount(async () => {
    isMaximized = await appWindow.isMaximized();

    const unlisten = await appWindow.onResized(async () => {
      isMaximized = await appWindow.isMaximized();
    });

    return () => {
      unlisten();
    };
  });

  async function saveMainWindowStateIfNeeded() {
    if (variant !== "main") return;
    try {
      await commandSaveMainWindowState();
    } catch (e) {
      console.error("Failed to save window state:", e);
    }
  }

  async function minimize() {
    console.log("minimize clicked");
    await saveMainWindowStateIfNeeded();
    appWindow.minimize().catch(console.error);
  }

  function toggleMaximize() {
    console.log("toggleMaximize clicked");
    appWindow.toggleMaximize().catch(console.error);
  }

  async function closeWindow() {
    console.log("close clicked");
    if (isDemoBuild && variant === "main") {
      push("/");
      return;
    }
    await saveMainWindowStateIfNeeded();
    appWindow.close().catch(console.error);
  }

  function startDragging(e: MouseEvent) {
    if (e.buttons === 1) {
      appWindow.startDragging().catch(console.error);
    }
  }
</script>

<div class="{heightClass} bg-bg-secondary flex justify-between items-center select-none w-full z-50 shrink-0 relative">
  <div class="flex items-center h-full">
    {#if variant === "main"}
      <div class="flex items-center justify-center h-full pl-2 pr-1 pointer-events-none">
        <img src={Icon} alt="launcherg icon" class="h-4" />
      </div>
      <a href={isDemoBuild ? "/demo" : "/"} use:link class="flex items-center px-2 h-full cursor-pointer outline-none border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium" tabindex="-1">
        {navLabels.home}
      </a>
      <APopover panelClass="left-0" let:close>
        <button slot="button" class="flex items-center px-2 h-full cursor-pointer outline-none focus:outline-none focus-visible:outline-none bg-transparent border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium">
          {navLabels.gameAdd}
        </button>
        <ImportPopover
          on:close={() => close(null)}
          on:startAuto={() => (isOpenImportAutomatically = true)}
          on:startManual={() => (isOpenImportManually = true)}
        />
      </APopover>

      <button on:click={openShortcutSettingsTab} class="flex items-center px-2 h-full cursor-pointer outline-none focus:outline-none focus-visible:outline-none bg-transparent border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium">
        {navLabels.shortcut}
      </button>

      <button on:click={openDisplaySettingsTab} class="flex items-center px-2 h-full cursor-pointer outline-none focus:outline-none focus-visible:outline-none bg-transparent border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium">
        {navLabels.display}
      </button>

      <button on:click={openSettingsTab} class="flex items-center px-2 h-full cursor-pointer outline-none focus:outline-none focus-visible:outline-none bg-transparent border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium">
        {navLabels.bulkEdit}
      </button>

      <a href="https://github.com/nnnSMM/Launcherg-Mod/blob/main/USAGE.md" target="_blank" rel="noopener noreferrer" class="flex items-center px-2 h-full cursor-pointer outline-none border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium" tabindex="-1">
        {navLabels.help}
      </a>
    {/if}
    <slot name="left" />
  </div>

  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div on:mousedown={startDragging} class="flex items-center justify-center h-full flex-1 cursor-default relative">
    <slot name="center" />
  </div>
  <div class="flex h-full items-center pr-[132px]">
    <slot name="right" />
  </div>

  <!-- window controls -->
  <div class="absolute top-0 right-0 flex {heightClass} items-start">
    <button tabindex="-1" on:click={minimize} class="w-11 {heightClass} flex items-center justify-center bg-transparent hover:bg-bg-tertiary transition-colors text-text-secondary hover:text-text-primary outline-none border-none focus:outline-none focus-visible:outline-none">
      <div class="i-material-symbols:remove text-lg"></div>
    </button>
    <button tabindex="-1" on:click={toggleMaximize} class="w-11 {heightClass} flex items-center justify-center bg-transparent hover:bg-bg-tertiary transition-colors text-text-secondary hover:text-text-primary outline-none border-none focus:outline-none focus-visible:outline-none">
      <div class={isMaximized ? "i-material-symbols:filter-none text-[14px]" : "i-material-symbols:check-box-outline-blank text-base"}></div>
    </button>
    <button tabindex="-1" on:click={closeWindow} class="w-11 {heightClass} flex items-center justify-center bg-transparent hover:bg-accent-error transition-colors text-text-secondary hover:text-text-white outline-none border-none focus:outline-none focus-visible:outline-none">
      <div class="i-material-symbols:close text-lg"></div>
    </button>
  </div>
</div>

{#if isOpenImportAutomatically}
  <ImportAutomatically bind:isOpen={isOpenImportAutomatically} />
{/if}
{#if isOpenImportManually}
  <ImportManually
    bind:isOpen={isOpenImportManually}
    on:confirm={(e) => importManually(e.detail)}
    on:cancel={() => (isOpenImportManually = false)}
  />
{/if}

<style>
  button, a {
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
    -webkit-user-drag: none;
    user-select: none;
  }
  button:focus, a:focus, button:active, a:active {
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
  }
</style>
