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

  import { localStorageWritable } from "@/lib/utils";
  const sidebarWidth = localStorageWritable("sidebar-width", 320);

  let isResizing = false;

  function startResize(e: MouseEvent) {
    isResizing = true;
    e.preventDefault();
  }

  function onMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    let newWidth = e.clientX;
    if (newWidth < 200) newWidth = 200; // min width
    if (newWidth > 800) newWidth = 800; // max width
    sidebarWidth.set(newWidth);
  }

  function onMouseUp() {
    isResizing = false;
  }
</script>

<svelte:window on:mousemove={onMouseMove} on:mouseup={onMouseUp} />

<div
  class="h-full min-h-0 relative border-(r-1px solid border-primary) glass !bg-bg-primary/40 flex-shrink-0 overflow-hidden"
  class:transition-all={!isResizing}
  style="width: {$showSidebar ? `${$sidebarWidth}px` : '3rem'};"
>
  {#if $showSidebar}
    <div class="absolute inset-0 min-w-0 flex flex-col" transition:fly={{ x: -40, duration: 150 }}>
      <div
        class="min-h-0 min-w-0 w-full flex-1 grid-(~ rows-[min-content_min-content_1fr])"
      >
        <div class="min-w-0 w-full">
          <SubHeader />
        </div>
        <div class="w-full mt-2 px-2 min-w-0">
          <Search
            bind:query={$query}
            bind:order={$currentSortOrder}
            playStatusAttributes={$playStatusAttributes}
            otherAttributes={$otherAttributes}
            on:toggleAttributeEnabled={(e) => toggleAttribute(e.detail.key)}
          />
        </div>
        <div class="mt-1 min-h-0 min-w-0 w-full">
          <CollectionElements
            collectionElement={$shown}
            on:update={() => sidebarCollectionElements.refetch()}
          />
        </div>
      </div>
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div
        class="absolute top-0 right-0 w-1.5 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors z-50 -mr-[1px]"
        on:mousedown={startResize}
      ></div>
    </div>
  {:else}
    <div class="absolute inset-0" transition:fly={{ x: 40, duration: 150 }}>
      <MinimalSidebar />
    </div>
  {/if}
</div>
