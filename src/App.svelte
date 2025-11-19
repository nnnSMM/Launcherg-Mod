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

  $: setDetailPromise = registerCollectionElementDetails();

  onMount(() => {
    initialize();
    initializeAllGameCache();
  });
</script>

<main class="relative h-full w-full bg-bg-primary font-sans overflow-hidden">
  {#if $backgroundState.imageUrl}
    <div
      transition:fade={{ duration: 300 }}
      class="absolute inset-0 bg-cover bg-center blur-2xl opacity-50 z-0"
      style="background-image: url({$backgroundState.imageUrl});"
    />
  {/if}
  <div class="relative h-full w-full z-10">
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
  </div>
</main>
