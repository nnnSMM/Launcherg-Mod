<script lang="ts">
  import Detail from "@/components/Work/Detail.svelte";
  import WorkImage from "@/components/Work/WorkImage.svelte";
  import WorkMain from "@/components/Work/WorkMain.svelte";
  import type { Work, CollectionElement } from "@/lib/types";
  import { commandGetCollectionElement } from "@/lib/command";

  export let work: Work;

  // Fetch the local collection element data. This contains the local thumbnail path and updatedAt timestamp.
  let elementPromise: Promise<CollectionElement> = commandGetCollectionElement(
    work.id
  );

  // When the image is updated in the child component, this function will be called to refetch the element data.
  const refetchElement = () => {
    elementPromise = commandGetCollectionElement(work.id);
  };
</script>

{#await elementPromise then element}
  <!-- Determine orientation based on the local thumbnail's dimensions -->
  {#if element.thumbnailWidth && element.thumbnailHeight && element.thumbnailWidth > element.thumbnailHeight}
    <div class="p-6 w-full min-h-0 max-w-192 space-y-6">
      <div class="w-full space-y-6">
        <!-- WorkImage uses the local `element` data -->
        <WorkImage {element} on:update={refetchElement} />
        <!-- WorkMain and Detail continue to use the scraped `work` data -->
        <WorkMain {work} />
      </div>
      <Detail {work} />
    </div>
  {:else}
    <div class="p-6 w-full min-h-0 max-w-256 space-y-6">
      <div
        class="grid grid-cols-[repeat(auto-fill,_minmax(320px,_1fr))] w-full gap-6"
      >
        <WorkImage {element} on:update={refetchElement} />
        <WorkMain {work} />
      </div>
      <Detail {work} />
    </div>
  {/if}
{:catch error}
    <p class="text-red-500">Error loading game data: {error.message}</p>
{/await}
