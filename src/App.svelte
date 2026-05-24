<script lang="ts">
  import Router from "svelte-spa-router";
  import Layout from "@/layouts/Layout.svelte";
  import { routes } from "@/router/route";
  import { initialize, routeLoaded } from "@/store/tabs";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { onMount } from "svelte";
  import { initializeAllGameCache } from "@/lib/scrapeAllGame";
  import ImportDropFiles from "@/components/Home/ImportDropFiles.svelte";
  import { backgroundState } from "@/store/background";
  import { location } from "svelte-spa-router";
  import { fade } from "svelte/transition";

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
    if (!isLandingRoute) {
      initializeMainApp();
      if (windowLabel === "main") {
        void appUpdate.initialize();
      }
    }

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
    class="relative h-full w-full bg-bg-primary font-sans overflow-hidden flex flex-col"
  >
    <TitleBar heightClass="h-8" />
    {#if $backgroundState.imageUrl}
      <div
        transition:fade={{ duration: 300 }}
        class="absolute inset-0 bg-cover bg-center blur-2xl opacity-50 z-0"
        style="background-image: url({$backgroundState.imageUrl});"
      />
    {/if}
    <div class="relative flex-1 w-full z-10 min-h-0">
      {#await setDetailPromise then _}
        <Layout>
          {#key $location}
            <div
              class="h-full w-full"
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
