<script lang="ts">
  import { onMount } from "svelte";
  import { derived } from "svelte/store";
  import CollectionElements from "@/components/Sidebar/CollectionElements.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { createWritable } from "@/lib/utils";
  import type { SortOrder } from "@/components/Sidebar/sort";
  import {
    type Option,
    collectionElementsToOptions,
    useFilter,
  } from "@/lib/filter";
  import Search from "@/components/Sidebar/Search.svelte";
  import Header from "@/components/Sidebar/Header.svelte";
  import { showSidebar } from "@/store/showSidebar";
  import MinimalSidebar from "@/components/Sidebar/MinimalSidebar.svelte";
  import { fly } from "svelte/transition";
  import SubHeader from "@/components/Sidebar/SubHeader.svelte";
  import {
    PLAY_STATUS_KEYS,
    type AttributeKey,
    type Attribute,
  } from "@/components/Sidebar/searchAttributes";
  import { search } from "@/components/Sidebar/search";
  import { query } from "@/store/query";
  import {
    currentSortOrder,
    currentAttributes,
    toggleAttribute,
  } from "@/store/viewSettings";

  onMount(async () => {
    await sidebarCollectionElements.refetch();
  });

  const [elementOptions, getElementOptions] = createWritable<Option<number>[]>(
    [],
  );
  sidebarCollectionElements.subscribe((v) =>
    elementOptions.set(collectionElementsToOptions(v)),
  );

  const { filtered } = useFilter(query, elementOptions, getElementOptions);

  const playStatusAttributes = derived(
    currentAttributes,
    ($attributes: Attribute[]) =>
      $attributes.filter((attr: Attribute) =>
        PLAY_STATUS_KEYS.includes(attr.key as AttributeKey),
      ),
  );
  const otherAttributes = derived(
    currentAttributes,
    ($attributes: Attribute[]) =>
      $attributes.filter(
        (attr: Attribute) =>
          !PLAY_STATUS_KEYS.includes(attr.key as AttributeKey),
      ),
  );

  const shown = sidebarCollectionElements.shown;

  $: shown.set(search($filtered, $currentAttributes, $currentSortOrder));
</script>

<div
  class="h-full min-h-0 relative border-(r-1px solid border-primary) transition-all glass !bg-bg-primary/40"
  class:w-80={$showSidebar}
  class:w-12={!$showSidebar}
>
  {#if $showSidebar}
    <div class="absolute inset-0" transition:fly={{ x: -40, duration: 150 }}>
      <div
        class="min-h-0 relative w-full h-full grid-(~ rows-[min-content_min-content_min-content_1fr])"
      >
        <Header />
        <SubHeader />
        <div class="w-full mt-2 px-2">
          <Search
            bind:query={$query}
            bind:order={$currentSortOrder}
            playStatusAttributes={$playStatusAttributes}
            otherAttributes={$otherAttributes}
            on:toggleAttributeEnabled={(e) => toggleAttribute(e.detail.key)}
          />
        </div>
        <div class="mt-1 min-h-0">
          <CollectionElements
            collectionElement={$shown}
            on:update={() => sidebarCollectionElements.refetch()}
          />
        </div>
      </div>
    </div>
  {:else}
    <div class="absolute inset-0" transition:fly={{ x: 40, duration: 150 }}>
      <MinimalSidebar />
    </div>
  {/if}
</div>
