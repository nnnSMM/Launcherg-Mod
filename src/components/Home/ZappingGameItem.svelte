<script lang="ts">
  import { link } from "svelte-spa-router";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  export let collectionElement: CollectionElement;

  $: imgSrc = convertFileSrc(collectionElement.thumbnail);
</script>

<!--
  - `hover:scale-115` is removed from the container.
  - `group` is added to control the hover state of child elements.
-->
<div
  class="group hover:shadow-md focus-within:shadow-md transition-shadow cursor-pointer w-full h-full relative hover:z-10"
>
  <a
    tabIndex={0}
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    use:link
    class="w-full h-full block"
  >
    {#if collectionElement.thumbnailWidth && collectionElement.thumbnailHeight}
      <!-- Base image. Stays put. -->
      <img
        decoding="async"
        class="object-contain rounded w-full h-full"
        src={imgSrc}
        alt={`${collectionElement.gamename}のサムネイル`}
      />
      <!--
        - Overlay image.
        - Uses the same hi-res source.
        - Is hidden by default (`opacity-0`).
        - On hover of the parent `group`, it fades in and scales up.
        - This scales only the image layer, which should prompt better rendering.
      -->
      <img
        decoding="async"
        class="absolute top-0 left-0 w-full h-full object-contain rounded opacity-0 group-hover:opacity-100 group-hover:scale-115 transition-all duration-300 ease-in-out"
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
