<script lang="ts">
  import type { CollectionElement } from "@/lib/types";
  import { formatLastPlayed } from "@/lib/utils";
  import ZappingGameItem from "./ZappingGameItem.svelte";

  export let collectionElement: CollectionElement;

  $: lastPlayedString = formatLastPlayed(collectionElement.lastPlayAt);
</script>

<div class="w-60 flex-shrink-0 space-y-1">
  <div class="text-sm text-text-secondary px-1">
    {lastPlayedString}
  </div>
  <div class="aspect-ratio-4/3">
    <ZappingGameItem {collectionElement} />
  </div>
</div>
