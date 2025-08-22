<script lang="ts">
  import { link } from "svelte-spa-router";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  export let collectionElement: CollectionElement;

  $: imgSrc = convertFileSrc(collectionElement.thumbnail);
</script>

<!--
  - Remove `hover:scale-115`
  - Add `group` to control children's hover state
-->
<div
  class="group hover:shadow-md focus-within:shadow-md transition-shadow cursor-pointer w-full h-full relative hover:z-10"
>
  <!--
    - Add `relative` for positioning context for the absolute image
    - Add `overflow-hidden` to clip the image neatly
    - Add flexbox properties to center the image
  -->
  <a
    tabIndex={0}
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    use:link
    class="w-full h-full block relative overflow-hidden flex items-center justify-center"
  >
    {#if collectionElement.thumbnailWidth && collectionElement.thumbnailHeight}
      <!--
        - The image is now absolutely positioned to not affect layout
        - On group-hover, its width and height are increased, forcing a re-render from the hi-res source
        - The transition is applied to width and height
      -->
      <img
        decoding="async"
        class="object-contain rounded w-full h-full absolute transition-all duration-300 ease-in-out group-hover:w-[115%] group-hover:h-[115%]"
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
