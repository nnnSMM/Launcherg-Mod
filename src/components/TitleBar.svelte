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
  import { autoImportProgress } from "@/store/autoImportProgress";
  import { showSidebar } from "@/store/showSidebar";
  import { canGoBack, canGoForward } from "@/lib/historyTrack";

  export let variant: "main" | "screenshot" = "main";
  export let heightClass: string = "h-8";

  const appWindow = getCurrentWindow();
  const isDemoBuild = import.meta.env.BASE_URL === "./";
  const navLabels = {
    back: "\u623b\u308b",
    forward: "\u9032\u3080",
    home: "\u30db\u30fc\u30e0",
    gameAdd: "\u30b2\u30fc\u30e0\u8ffd\u52a0",
    addShort: "\u8ffd\u52a0",
    shortcut: "\u30b7\u30e7\u30fc\u30c8\u30ab\u30c3\u30c8",
    display: "\u8868\u793a",
    bulkEdit: "\u30d7\u30ec\u30a4\u72b6\u614b\u4e00\u62ec\u7de8\u96c6",
    help: "\u30d8\u30eb\u30d7",
    toggleSidebar: "\u30b5\u30a4\u30c9\u30d0\u30fc\u306e\u5207\u308a\u66ff\u3048",
  };
  const demoRegistrationDisabledMessage =
    "demo \u3067\u306f\u30b2\u30fc\u30e0\u767b\u9332\u306f\u3067\u304d\u307e\u305b\u3093\u3002\u30db\u30fc\u30e0\u306e\u300c\u30d5\u30a9\u30eb\u30c0\u7d10\u3065\u3051\u3092\u8a66\u3059\u300d\u3067\u5224\u5b9a\u3060\u3051\u78ba\u8a8d\u3067\u304d\u307e\u3059\u3002";

  let isMaximized = false;
  let isOpenImportAutomatically = false;
  let isOpenImportManually = false;

  const titlebarIconButtonClass =
    "h-8 w-8 flex items-center justify-center rounded-md cursor-pointer outline-none focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-accent bg-transparent border border-transparent text-text-secondary hover:bg-bg-tertiary hover:text-text-primary transition-colors";
  const titlebarActionButtonClass =
    "h-8 px-3 flex items-center gap-1.5 rounded-md cursor-pointer outline-none focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-accent bg-accent-primary/12 border border-accent-primary/40 text-text-primary hover:bg-accent-primary/24 transition-colors text-[12px] font-medium whitespace-nowrap";
  const titlebarToolButtonClass =
    "h-8 px-2 xl:px-2.5 flex items-center gap-1.5 rounded-md cursor-pointer outline-none focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-accent bg-transparent border border-transparent text-text-secondary hover:bg-bg-tertiary hover:text-text-primary transition-colors text-[12px] font-medium whitespace-nowrap";
  const titlebarDividerClass = "h-5 w-px bg-border-primary/80 mx-1";

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

  function goBack() {
    window.history.back();
  }

  function goForward() {
    window.history.forward();
  }
</script>

<div class="{heightClass} bg-bg-primary/92 border-b border-solid border-border-primary flex items-center select-none w-full z-50 shrink-0 relative backdrop-blur-xl">
  <div class="flex h-full min-w-0 flex-1 items-center pr-[132px]">
    <div class="flex items-center h-full gap-1 pl-1.5 pr-2 shrink-0">
      {#if variant === "main"}
        <button
          type="button"
          aria-label={navLabels.toggleSidebar}
          title={navLabels.toggleSidebar}
          class={titlebarIconButtonClass}
          on:click={() => showSidebar.update((v) => !v)}
        >
          <div class={$showSidebar ? "i-material-symbols:left-panel-close-outline text-[18px]" : "i-material-symbols:left-panel-open-outline text-[18px]"} />
        </button>
        <button
          type="button"
          aria-label={navLabels.back}
          title={navLabels.back}
          class={`${titlebarIconButtonClass} disabled:opacity-30 disabled:pointer-events-none`}
          disabled={!$canGoBack}
          on:click={goBack}
        >
          <div class="i-material-symbols:arrow-back-rounded text-[18px]" />
        </button>
        <button
          type="button"
          aria-label={navLabels.forward}
          title={navLabels.forward}
          class={`${titlebarIconButtonClass} disabled:opacity-30 disabled:pointer-events-none`}
          disabled={!$canGoForward}
          on:click={goForward}
        >
          <div class="i-material-symbols:arrow-forward-rounded text-[18px]" />
        </button>
        <a
          href={isDemoBuild ? "/demo" : "/"}
          use:link
          aria-label={navLabels.home}
          title={navLabels.home}
          class={titlebarIconButtonClass}
          tabindex="-1"
        >
          <div class="i-material-symbols:home-outline-rounded text-[18px]" />
        </a>
        <div class={titlebarDividerClass} />
        <APopover panelClass="left-0" let:close>
          <button
            slot="button"
            type="button"
            aria-label={navLabels.gameAdd}
            title={navLabels.gameAdd}
            class={titlebarActionButtonClass}
          >
            <div class="i-material-symbols:add-rounded text-[18px]" />
            <span>{navLabels.addShort}</span>
          </button>
          <ImportPopover
            on:close={() => close(null)}
            on:startAuto={() => (isOpenImportAutomatically = true)}
            on:startManual={() => (isOpenImportManually = true)}
          />
        </APopover>
      {/if}
      <slot name="left" />
    </div>

    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div on:mousedown={startDragging} class="flex h-full min-w-0 flex-1 items-center justify-center cursor-default relative">
      <slot name="center" />
    </div>

    <div class="flex h-full min-w-0 shrink-0 items-center gap-1 px-2">
      {#if variant === "main"}
        <div class="{titlebarDividerClass} hidden sm:block" />
        <button
          type="button"
          aria-label={navLabels.shortcut}
          title={navLabels.shortcut}
          class={titlebarToolButtonClass}
          on:click={openShortcutSettingsTab}
        >
          <div class="i-material-symbols:keyboard-outline-rounded text-[18px]" />
          <span class="hidden xl:inline">{navLabels.shortcut}</span>
        </button>
        <button
          type="button"
          aria-label={navLabels.display}
          title={navLabels.display}
          class={titlebarToolButtonClass}
          on:click={openDisplaySettingsTab}
        >
          <div class="i-material-symbols:palette-outline-rounded text-[18px]" />
          <span class="hidden xl:inline">{navLabels.display}</span>
        </button>
        <button
          type="button"
          aria-label={navLabels.bulkEdit}
          title={navLabels.bulkEdit}
          class={titlebarToolButtonClass}
          on:click={openSettingsTab}
        >
          <div class="i-material-symbols:fact-check-outline-rounded text-[18px]" />
          <span class="hidden 2xl:inline">{navLabels.bulkEdit}</span>
        </button>
        <a
          href="https://github.com/nnnSMM/Launcherg-Mod/blob/main/USAGE.md"
          target="_blank"
          rel="noopener noreferrer"
          aria-label={navLabels.help}
          title={navLabels.help}
          class={titlebarIconButtonClass}
          tabindex="-1"
        >
          <div class="i-material-symbols:help-outline-rounded text-[18px]" />
        </a>
      {/if}
      <slot name="right" />
    </div>
  </div>

  <!-- window controls -->
  <div class="absolute top-0 right-0 flex {heightClass} items-start border-l border-border-primary">
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

{#if $autoImportProgress.isRunning}
  <div class="fixed top-10 right-3 z-[60] rounded-md border border-border-primary bg-bg-secondary/95 px-3 py-2 shadow-lg backdrop-blur-sm">
    <div class="flex items-center gap-3">
      <div class="h-4 w-4 rounded-full border-2 border-border-primary border-t-accent-accent animate-spin" />
      <div class="min-w-0">
        <div class="text-[12px] font-medium text-text-primary">
          {$autoImportProgress.message}
        </div>
        {#if $autoImportProgress.total > 0}
          <div class="text-[11px] text-text-tertiary">
            {$autoImportProgress.processed}/{$autoImportProgress.total}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

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
    -webkit-user-drag: none;
    user-select: none;
  }
</style>
