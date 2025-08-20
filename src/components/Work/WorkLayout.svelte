<script lang="ts">
  import Detail from "@/components/Work/Detail.svelte";
  import WorkImage from "@/components/Work/WorkImage.svelte";
  import WorkMain from "@/components/Work/WorkMain.svelte";
  import type { Work, CollectionElement } from "@/lib/types";
  import { commandGetCollectionElement } from "@/lib/command";

  export let work: Work;

  let elementPromise: Promise<CollectionElement> = commandGetCollectionElement(
    work.id
  );

  const refetch = () => {
    elementPromise = commandGetCollectionElement(work.id);
  };
</script>

{#await elementPromise then element}
  {#if element.thumbnailWidth && element.thumbnailHeight && element.thumbnailWidth > element.thumbnailHeight}
    <div class="p-(x-8 y-8) w-full min-h-0 max-w-192 space-y-8">
      <div class="w-full space-y-8">
        <WorkImage {element} on:update={refetch} />
        <WorkMain {work} />
      </div>
      <Detail {work} />
    </div>
  {:else}
    <div class="p-(x-8 y-8) w-full min-h-0 max-w-256 space-y-8">
      <div
        class="grid grid-cols-[repeat(auto-fill,_minmax(320px,_1fr))] w-full gap-8"
      >
        <WorkImage {element} on:update={refetch} />
        <WorkMain {work} />
      </div>
      <Detail {work} />
    </div>
  {/if}
{/await}
