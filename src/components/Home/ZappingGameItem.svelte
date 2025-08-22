<script lang="ts">
  import { link } from "svelte-spa-router";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { works } from "@/store/works";

  export let collectionElement: CollectionElement;

  $: imgSrc = convertFileSrc(collectionElement.thumbnail);

  // State for high-resolution image
  let highResImgSrc = "";
  let isHighResLoading = false;

  // Function to load high-resolution image on hover
  const loadHighResImage = async () => {
    if (highResImgSrc || isHighResLoading) {
      return;
    }
    isHighResLoading = true;
    try {
      const work = await works.get(collectionElement.id);
      if (work?.imgUrl) {
        highResImgSrc = work.imgUrl;
      }
    } catch (e) {
      console.error(e);
    } finally {
      isHighResLoading = false;
    }
  };
</script>

<!-- Add 'group' class to enable group-hover functionality -->
<div
  class="group hover:scale-115 hover:shadow-md focus-within:scale-110 focus-within:shadow-md transition-all cursor-pointer w-full h-full relative hover:z-10"
  on:mouseenter={loadHighResImage}
>
  <a
    tabIndex={0}
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    use:link
    class="w-full h-full block"
  >
    {#if collectionElement.thumbnailWidth && collectionElement.thumbnailHeight}
      <!-- Low-resolution thumbnail (always visible as a base) -->
      <img
        decoding="async"
        class="object-contain rounded w-full h-full"
        src={imgSrc}
        alt={`${collectionElement.gamename}のサムネイル`}
      />

      <!-- High-resolution image (loads and fades in on hover) -->
      {#if highResImgSrc}
        <img
          src={highResImgSrc}
          alt={`${collectionElement.gamename}の高画質サムネイル`}
          class="object-contain rounded w-full h-full absolute top-0 left-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300"
        />
      {/if}
    {:else}
      <div
        class="text-(body text-primary) font-bold px-6 rounded border bg-bg-primary w-full h-full flex items-center justify-center"
      >
        {collectionElement.gamename}
      </div>
    {/if}
  </a>
</div>
