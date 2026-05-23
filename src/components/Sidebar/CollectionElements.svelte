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

  const emptyLabels = {
    title: "\u8868\u793a\u3067\u304d\u308b\u30b2\u30fc\u30e0\u304c\u3042\u308a\u307e\u305b\u3093",
    body: "\u691c\u7d22\u6761\u4ef6\u3084\u30d5\u30a3\u30eb\u30bf\u3092\u898b\u76f4\u3057\u3066\u304f\u3060\u3055\u3044",
  };
</script>

<div use:simplebar class="h-full overflow-y-auto">
  {#if collectionElement.length}
    <div class="w-full pt-2 pb-3">
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
  {:else}
    <div class="flex h-full min-h-48 flex-col items-center justify-center px-6 text-center">
      <div
        class="mb-3 grid h-10 w-10 place-items-center rounded-lg border border-border-primary bg-bg-primary/30 text-text-tertiary"
      >
        <div class="i-material-symbols:search-off-rounded h-5 w-5" />
      </div>
      <div class="text-sm font-medium text-text-secondary">
        {emptyLabels.title}
      </div>
      <div class="mt-1 text-xs leading-relaxed text-text-tertiary">
        {emptyLabels.body}
      </div>
    </div>
  {/if}
</div>
