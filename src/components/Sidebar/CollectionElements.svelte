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

<div class="h-full">
  {#if collectionElement.length}
    <div class="flex-1 mt-2 min-h-0 h-full">
      <div use:simplebar class="h-full overflow-y-auto">
        <div class="w-full">
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
      </div>
    </div>
  {/if}
</div>
