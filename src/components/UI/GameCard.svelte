<script lang="ts">
  import GameHoverPreview from "@/components/UI/GameHoverPreview.svelte";
  import { loadGamePreviewScreenshots } from "@/lib/useGameScreenshots";
  import type { CollectionElement, GameScreenshot } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { link } from "svelte-spa-router";

  export let collectionElement: CollectionElement;

  $: src = `${convertFileSrc(collectionElement.thumbnail)}?v=${
    collectionElement.updatedAt
  }`;

  let hoverNode: HTMLAnchorElement;
  let hoverTimer: ReturnType<typeof setTimeout> | null = null;
  let isPreviewVisible = false;
  let previewScreenshots: GameScreenshot[] = [];
  let hoverToken = 0;

  const closePreview = () => {
    hoverToken += 1;
    isPreviewVisible = false;
    if (hoverTimer) {
      clearTimeout(hoverTimer);
      hoverTimer = null;
    }
  };

  const openPreview = () => {
    closePreview();
    const token = hoverToken;
    hoverTimer = setTimeout(async () => {
      isPreviewVisible = true;
      previewScreenshots = [];
      const screenshots = await loadGamePreviewScreenshots(collectionElement);
      if (token === hoverToken) {
        previewScreenshots = screenshots;
      }
    }, 500);
  };
</script>

<div class="w-full h-full relative">
  <a
    bind:this={hoverNode}
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    use:link
    on:mouseenter={openPreview}
    on:mouseleave={closePreview}
    on:focus={openPreview}
    on:blur={closePreview}
    class="block w-full h-full relative group overflow-hidden rounded-xl shadow-lg transition-all duration-300 hover:shadow-xl hover:-translate-y-1"
  >
    <div class="w-full h-full bg-bg-secondary">
      {#if collectionElement.thumbnail}
        <img
          {src}
          alt={collectionElement.gamename}
          class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
          loading="lazy"
          on:error={(e) => {
            const img = e.currentTarget;
            if (img instanceof HTMLImageElement) {
              img.src = "/images/dummy_thumbnail.svg";
            }
          }}
        />
      {:else}
        <div class="w-full h-full flex items-center justify-center bg-bg-secondary">
          <img
            src="/images/dummy_thumbnail.svg"
            alt=""
            class="w-full h-full object-cover opacity-50"
          />
        </div>
      {/if}
    </div>

    <div
      class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"
    />

    <div
      class="absolute bottom-0 left-0 right-0 p-4 translate-y-full group-hover:translate-y-0 transition-transform duration-300"
    >
      <div class="text-white font-bold text-sm line-clamp-2 drop-shadow-md">
        {collectionElement.gamename}
      </div>
    </div>

    {#if collectionElement.playStatus === 2}
      <div
        class="absolute top-2 right-2 px-2 py-1 rounded bg-accent-success/90 text-white text-xs font-bold opacity-0 group-hover:opacity-100 transition-opacity"
      >
        CLEAR
      </div>
    {/if}
  </a>

  {#if isPreviewVisible}
    <GameHoverPreview
      {collectionElement}
      screenshots={previewScreenshots}
      anchorElement={hoverNode}
    />
  {/if}
</div>
