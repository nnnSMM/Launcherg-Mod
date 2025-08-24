<script lang="ts">
  import Detail from "@/components/Work/Detail.svelte";
  import WorkImage from "@/components/Work/WorkImage.svelte";
  import WorkMain from "@/components/Work/WorkMain.svelte";
  import type { Work, CollectionElement } from "@/lib/types";
  import { commandGetCollectionElement } from "@/lib/command";

  export let work: Work;
  export let element: CollectionElement;
  export let scrollY: number;

  // When the image is updated in the child component, this function will be called to refetch the element data.
  const refetchElement = () => {
    // This function will need to be updated to refetch at a higher level,
    // but for now, we'll just make it a no-op to avoid breaking things.
    // A full implementation would involve passing an update function down.
  };
</script>

<!-- Determine orientation based on the local thumbnail's dimensions -->
{#if element.thumbnailWidth && element.thumbnailHeight && element.thumbnailWidth > element.thumbnailHeight}
    <div class="p-(x-8 y-8) w-full min-h-0 max-w-192 space-y-8">
      <div class="w-full space-y-8">
        <!-- WorkImage uses the local `element` data -->
        <WorkImage {element} {scrollY} on:update={refetchElement} />
        <!-- WorkMain and Detail continue to use the scraped `work` data -->
        <WorkMain {work} />
      </div>
      <Detail {work} />
    </div>
  {:else}
    <div class="p-(x-8 y-8) w-full min-h-0 max-w-256 space-y-8">
      <div
        class="grid grid-cols-[repeat(auto-fill,_minmax(320px,_1fr))] w-full gap-8"
      >
        <WorkImage {element} {scrollY} on:update={refetchElement} />
        <WorkMain {work} />
      </div>
      <Detail {work} />
    </div>
  {/if}
