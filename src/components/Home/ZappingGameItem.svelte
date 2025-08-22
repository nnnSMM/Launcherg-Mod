<script lang="ts">
  import { link } from "svelte-spa-router";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  export let collectionElement: CollectionElement;

  $: imgSrc = convertFileSrc(collectionElement.thumbnail);
</script>

<!--
  - The container now scales on hover (`hover:scale-115`).
  - It also has `group` to control child hover states.
-->
<div
  class="group hover:scale-[1.125] hover:shadow-md focus-within:scale-110 focus-within:shadow-md transition-all cursor-pointer w-full h-full relative hover:z-10"
>
  <!-- The link needs to be relative to position the absolute overlay -->
  <a
    tabIndex={0}
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    use:link
    class="w-full h-full block relative"
  >
    {#if collectionElement.thumbnailWidth && collectionElement.thumbnailHeight}
      <!-- Base image. This will be scaled with the parent div. -->
      <img
        decoding="async"
        class="object-contain rounded w-full h-full"
        src={imgSrc}
        alt={`${collectionElement.gamename}のサムネイル`}
      />
      <!--
        - Overlay image, using the same hi-res source.
        - It is positioned directly on top of the base image.
        - On hover, it fades in (`group-hover:opacity-100`).
        - Since it's inside the scaling container, it will also be scaled up.
        - The theory is that fading in a new element forces the browser to re-render it sharply at the new, scaled size.
      -->
      <img
        decoding="async"
        class="absolute top-0 left-0 w-full h-full object-contain rounded opacity-0 group-hover:opacity-100 transition-opacity duration-300"
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
