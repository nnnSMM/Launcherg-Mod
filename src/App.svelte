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
  import { fade } from "svelte/transition";
  import { invoke } from "@tauri-apps/api/core";
  import { register, isRegistered } from "@tauri-apps/plugin-global-shortcut";

  $: setDetailPromise = registerCollectionElementDetails();

  async function launchGame() {
    await invoke("launch_shortcut_game");
  }

  onMount(async () => {
    initialize();
    initializeAllGameCache();

    const savedShortcutKey = await invoke<string>("get_app_setting", {
      key: "shortcut_key",
    });
    if (savedShortcutKey) {
      if (!(await isRegistered(savedShortcutKey))) {
        await register(savedShortcutKey, launchGame);
      }
    }
  });
</script>

<main class="relative h-full w-full bg-bg-primary font-sans">
  {#if $backgroundState.imageUrl}
    <div
      transition:fade={{ duration: 300 }}
      class="absolute inset-0 bg-cover bg-top blur-xl"
      style="background-image: url({$backgroundState.imageUrl}); opacity: {$backgroundState.opacity};"
    />
  {/if}
  <div class="relative h-full w-full">
    {#await setDetailPromise then _}
      <Layout>
        <Router {routes} on:routeLoaded={routeLoaded} />
      </Layout>
    {/await}
    <ImportDropFiles />
  </div>
</main>
