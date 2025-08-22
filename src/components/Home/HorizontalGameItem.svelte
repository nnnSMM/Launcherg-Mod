<script lang="ts">
  import type { CollectionElement } from "@/lib/types";
  import { formatLastPlayed } from "@/lib/utils";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { link } from "svelte-spa-router";

  export let collectionElement: CollectionElement;

  $: lastPlayedString = formatLastPlayed(collectionElement.lastPlayAt);
  $: imgSrc = convertFileSrc(collectionElement.thumbnail);
</script>

<div class="w-80 flex-shrink-0 space-y-1">
  <div class="text-sm text-text-secondary px-1 truncate">
    {lastPlayedString}
  </div>
  <div class="aspect-ratio-4/3">
    <!-- This is a simplified version of ZappingGameItem -->
    <div
      class="group hover:shadow-md focus-within:shadow-md transition-shadow cursor-pointer w-full h-full relative hover:z-10"
    >
      <a
        tabIndex={0}
        href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
        use:link
        class="w-full h-full block relative"
      >
        {#if collectionElement.thumbnail}
          <!-- Base image -->
          <img
            decoding="async"
            class="object-cover rounded w-full h-full"
            src={imgSrc}
            alt={`${collectionElement.gamename}のサムネイル`}
          />
          <!-- Overlay image for hover effect -->
          <img
            decoding="async"
            class="absolute top-0 left-0 w-full h-full object-cover rounded opacity-0 group-hover:opacity-100 group-hover:scale-105 transition-all duration-300 ease-in-out"
            src={imgSrc}
            alt={`${collectionElement.gamename}のサムネイル`}
          />
        {:else}
          <div
            class="text-(body text-primary) font-bold px-6 rounded border bg-bg-primary w-full h-full flex items-center justify-center"
          >
            {collectionElement.gamename}
          </div>
        {/if}
      </a>
    </div>
  </div>
</div>
