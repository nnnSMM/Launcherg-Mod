<script lang="ts">
  import { link } from "svelte-spa-router";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";

  export let collectionElement: CollectionElement;
  export let objectFit: "contain" | "cover" = "contain";
  export let containerClass = "";
</script>

<div class={`w-full h-full ${containerClass}`}>
  <slot name="info" />
  <div
    class="group hover:scale-[1.025] hover:shadow-md focus-within:scale-110 focus-within:shadow-md transition-all cursor-pointer w-full h-full relative hover:z-10"
  >
    <a
      tabIndex={0}
      href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
      use:link
      class="w-full h-full block relative"
    >
      {#if collectionElement.thumbnail}
        <img
          decoding="async"
          class="rounded w-full h-full"
          class:object-contain={objectFit === "contain"}
          class:object-cover={objectFit === "cover"}
          src={convertFileSrc(collectionElement.thumbnail)}
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
