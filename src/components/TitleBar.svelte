<script lang="ts">
  export let variant: "main" | "screenshot" = "main";
  export let heightClass: string = "h-8";
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Icon from "/icon.png";
  import { link } from "svelte-spa-router";
  import { openSettingsTab, openShortcutSettingsTab } from "@/store/tabs";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import APopover from "@/components/UI/APopover.svelte";
  import ImportPopover from "@/components/Sidebar/ImportPopover.svelte";
  import ImportAutomatically from "@/components/Sidebar/ImportAutomatically.svelte";
  import ImportManually from "@/components/Sidebar/ImportManually.svelte";
  import { commandUpsertCollectionElement } from "@/lib/command";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { showErrorToast, showInfoToast } from "@/lib/toast";
  import type { AllGameCacheOne } from "@/lib/types";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";

  const appWindow = getCurrentWindow();
  let isMaximized = false;
  let isOpenImportAutomatically = false;
  let isOpenImportManually = false;

  const importManually = async (arg: {
    exePath: string | null;
    lnkPath: string | null;
    gameCache: AllGameCacheOne;
  }) => {
    try {
      await commandUpsertCollectionElement(arg);
      try {
        await registerCollectionElementDetails();
      } catch (e) {
        console.error("Failed to fetch extended game details:", e);
      }
      showInfoToast(`${arg.gameCache.gamename}を登録しました。`);
    } catch (e) {
      console.error("Failed to add game to collection:", e);
      showInfoToast(`${arg.gameCache.gamename}を登録しました。`);
    } finally {
      await sidebarCollectionElements.refetch();
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

<div class="{heightClass} bg-bg-secondary flex justify-between items-center select-none w-full z-50 shrink-0 relative">
  <div class="flex items-center h-full">
    {#if variant === "main"}
      <div class="flex items-center justify-center h-full pl-2 pr-1 pointer-events-none">
        <img src={Icon} alt="launcherg icon" class="h-4" />
      </div>
      <a href="/" use:link class="flex items-center px-2 h-full cursor-pointer outline-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium" tabindex="-1">
        ホーム
      </a>
      
      <APopover panelClass="left-0" let:close>
        <button slot="button" class="flex items-center px-2 h-full cursor-pointer outline-none focus:outline-none focus-visible:outline-none bg-transparent border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium">
          ゲーム追加
        </button>
        <ImportPopover
          on:close={() => close(null)}
          on:startAuto={() => (isOpenImportAutomatically = true)}
          on:startManual={() => (isOpenImportManually = true)}
        />
      </APopover>

      <button on:click={openShortcutSettingsTab} class="flex items-center px-2 h-full cursor-pointer outline-none focus:outline-none focus-visible:outline-none bg-transparent border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium">
        ショートカット
      </button>

      <button on:click={openSettingsTab} class="flex items-center px-2 h-full cursor-pointer outline-none focus:outline-none focus-visible:outline-none bg-transparent border-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium">
        一括編集
      </button>

      <a href="https://github.com/nnnSMM/Launcherg-Mod/blob/main/USAGE.md" target="_blank" rel="noopener noreferrer" class="flex items-center px-2 h-full cursor-pointer outline-none text-text-secondary hover:text-text-primary transition-colors text-[13px] font-medium" tabindex="-1">
        ヘルプ
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
  <div class="absolute top-0 right-0 flex h-8 items-start">
    <button tabindex="-1" on:click={minimize} class="w-11 h-8 flex items-center justify-center bg-transparent hover:bg-bg-tertiary transition-colors text-text-secondary hover:text-text-primary outline-none border-none focus:outline-none focus-visible:outline-none">
      <div class="i-material-symbols:remove text-lg"></div>
    </button>
    <button tabindex="-1" on:click={toggleMaximize} class="w-11 h-8 flex items-center justify-center bg-transparent hover:bg-bg-tertiary transition-colors text-text-secondary hover:text-text-primary outline-none border-none focus:outline-none focus-visible:outline-none">
      <div class={isMaximized ? "i-material-symbols:filter-none text-[14px]" : "i-material-symbols:check-box-outline-blank text-base"}></div>
    </button>
    <button tabindex="-1" on:click={closeWindow} class="w-11 h-8 flex items-center justify-center bg-transparent hover:bg-accent-error transition-colors text-text-secondary hover:text-text-white outline-none border-none focus:outline-none focus-visible:outline-none">
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
