<script lang="ts">
  import { onMount } from "svelte";
  import { works } from "@/store/works";
  import Work from "@/components/Work/Work.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { commandGetCollectionElement } from "@/lib/command";
  import type { CollectionElement, Work as WorkType } from "@/lib/types";

  export let params: { id: string };

  onMount(() => {
    sidebarCollectionElements.refetch();
  });

  $: workAndElementPromise = (async () => {
    const work = await works.get(+params.id);
    if (!work) {
      throw new Error("Work not found");
    }
    const element = await commandGetCollectionElement(work.id);
    return { work, element };
  })();
</script>

{#await workAndElementPromise then { work, element }}
  <div class="w-full h-full">
    <Work {work} {element} />
  </div>
{:catch error}
  <div class="p-4 text-text-error">
    Error loading game data: {error.message}
  </div>
{/await}
