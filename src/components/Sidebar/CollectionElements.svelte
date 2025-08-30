<script lang="ts">
  import CollectionElement from "@/components/Sidebar/CollectionElement.svelte";
  import type { CollectionElementsWithLabel } from "@/lib/types";
  import SimpleBar from "simplebar";
  import ADisclosure from "@/components/UI/ADisclosure.svelte";
  import { createEventDispatcher } from "svelte";

  export let collectionElement: CollectionElementsWithLabel[];
  const dispatcher = createEventDispatcher();

  const simplebar = (node: HTMLElement) => {
    new SimpleBar(node, { scrollbarMinSize: 64 });
  };
</script>

<div use:simplebar class="h-full overflow-y-auto">
  {#if collectionElement.length}
    <div class="w-full pt-2">
      {#each collectionElement as { label, elements } (label)}
        <ADisclosure {label} defaultOpen={collectionElement.length === 1}>
          {#each elements as ele (ele.id)}
            <CollectionElement
              collectionElement={ele}
              on:update={() => dispatcher("update")}
            />
          {/each}
        </ADisclosure>
      {/each}
    </div>
  {/if}
</div>
