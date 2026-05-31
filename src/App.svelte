<script lang="ts">
  import Router from "svelte-spa-router";
  import Layout from "@/layouts/Layout.svelte";
  import { routes } from "@/router/route";
  import { initialize, routeLoaded, syncSelectedToLocation } from "@/store/tabs";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { onMount } from "svelte";
  import { setupGlobalTooltips } from "@/lib/tooltip";
  import { setupHistoryTracker } from "@/lib/historyTrack";
  import { initializeAllGameCache } from "@/lib/scrapeAllGame";
  import ImportDropFiles from "@/components/Home/ImportDropFiles.svelte";
  import { backgroundState } from "@/store/background";
  import { location, replace } from "svelte-spa-router";
  import { fade } from "svelte/transition";
  import { isWorkDetailRoute } from "@/lib/routeHelper";

  $: isWorkDetail = isWorkDetailRoute($location);

  import { getCurrentWindow } from "@tauri-apps/api/window";

  import Overlay from "@/views/Overlay.svelte";
  import Landing from "@/views/Landing.svelte";
  import ScreenshotWindow from "@/views/ScreenshotWindow.svelte";
  import TrayMenu from "@/views/TrayMenu.svelte";
  import InitializationOverlay from "@/components/UI/InitializationOverlay.svelte";
  import TitleBar from "@/components/TitleBar.svelte";
  import { theme } from "@/store/theme";
  import { appUpdate } from "@/store/update";

  const windowLabel = getCurrentWindow().label;
  const isPublicDemoBuild = __PUBLIC_DEMO_BUILD__;
  let didInitializeMainApp = false;
  let isMounted = false;

  $: isLandingRoute =
    isPublicDemoBuild && ($location === "/" || $location === "/landing");
  $: setDetailPromise = isLandingRoute
    ? Promise.resolve()
    : registerCollectionElementDetails();
  $: if ($location === "/settings/display") {
    replace("/settings/shortcut");
  }
  $: if (!isLandingRoute) {
    syncSelectedToLocation($location);
  }

  const handleMainScroll = (e: Event) => {
    const target = e.currentTarget as HTMLElement;
    if (target.scrollTop !== 0) {
      target.scrollTop = 0;
    }
  };

  const initializeMainApp = () => {
    if (didInitializeMainApp) {
      return;
    }
    didInitializeMainApp = true;
    initialize();
    initializeAllGameCache();
  };

  onMount(async () => {
    void theme.initialize();
    isMounted = true;
    // 以前の自動生成デモ用サンプルメモが残っている場合はクリーンアップ（削除）する
    const demoIds = [39837, 27059, 38696, 38631, 26245, 28941, 30122, 25861, 20988, 31106, 31597, 38794];
    demoIds.forEach((id) => {
      const key = `smde_memo-${id}`;
      const val = localStorage.getItem(key);
      if (val && (val.includes("自動生成サンプルメモ") || val.includes("攻略進捗とメモ"))) {
        localStorage.removeItem(key);
      }
    });

    if (!isLandingRoute) {
      initializeMainApp();

      if (windowLabel === "main") {
        void appUpdate.initialize();
      }
    }


    const cleanupTooltips = setupGlobalTooltips();
    const cleanupHistory = setupHistoryTracker();

    // フォーカス取得時のWebView自動スクロールを防止
    const handleFocusIn = () => {
      setTimeout(() => {
        if (window.scrollY !== 0) {
          window.scrollTo(0, 0);
        }
        const mainElement = document.querySelector("main");
        if (mainElement && mainElement.scrollTop !== 0) {
          mainElement.scrollTop = 0;
        }
      }, 0);
    };
    document.addEventListener("focusin", handleFocusIn);

    const preventScroll = () => {
      if (window.scrollY !== 0) {
        window.scrollTo(0, 0);
      }
    };
    window.addEventListener("scroll", preventScroll, { passive: true });

    // F5とCtrl+Rによるリロードを無効化
    const handleKeydown = (e: KeyboardEvent) => {
      if (
        e.key === "F5" ||
        (e.ctrlKey && e.key === "r") ||
        (e.metaKey && e.key === "r")
      ) {
        e.preventDefault();
      }
    };
    window.addEventListener("keydown", handleKeydown);

    return () => {
      window.removeEventListener("keydown", handleKeydown);
      document.removeEventListener("focusin", handleFocusIn);
      window.removeEventListener("scroll", preventScroll);
      cleanupTooltips();
      cleanupHistory();
    };
  });

  $: if (isMounted && !isLandingRoute) {
    initializeMainApp();
    if (windowLabel === "main") {
      void appUpdate.initialize();
    }
  }
</script>

{#if windowLabel === "overlay"}
  <Overlay />
{:else if windowLabel === "screenshot_window"}
  <ScreenshotWindow />
{:else if windowLabel === "tray_menu"}
  <TrayMenu />
{:else if isLandingRoute}
  <Landing />
{:else}
  <main
    class="relative h-full w-full font-sans overflow-hidden flex flex-col transition-colors duration-300"
    class:bg-bg-primary={!isWorkDetail}
    class:bg-transparent={isWorkDetail}
    on:scroll={handleMainScroll}
  >
    <TitleBar heightClass="h-10" />
    {#if $backgroundState.imageUrl}
      {#key $backgroundState.imageUrl}
        <div
          transition:fade={{ duration: 300 }}
          class="absolute inset-0 bg-cover bg-center z-0 transition-all duration-500 ease-out transform"
          class:opacity-85={isWorkDetail}
          class:scale-100={isWorkDetail}
          class:blur-2xl={!isWorkDetail}
          class:opacity-50={!isWorkDetail}
          style="background-image: url('{$backgroundState.imageUrl}'); {isWorkDetail ? 'filter: blur(64px) brightness(1.0);' : ''}"
        />
      {/key}
    {/if}
    <div class="relative flex-1 w-full z-10 min-h-0">
      {#await setDetailPromise then _}
        <Layout>
          {#key $location}
            <div
              class="absolute inset-0"
              in:fade={{ duration: 200, delay: 200 }}
              out:fade={{ duration: 200 }}
            >
              <Router {routes} on:routeLoaded={routeLoaded} />
            </div>
          {/key}
        </Layout>
      {/await}
      <ImportDropFiles />
      <InitializationOverlay />
    </div>
  </main>
{/if}
