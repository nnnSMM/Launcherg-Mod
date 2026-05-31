<script lang="ts">
  import type { CollectionElement, GameScreenshot } from "@/lib/types";
  import { formatLastPlayed, formatPlayTime } from "@/lib/utils";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";

  export let collectionElement: CollectionElement;
  export let screenshots: GameScreenshot[] = [];
  export let anchorElement: HTMLElement | null = null;

  const WIDTH = 440;
  const HEIGHT = 286;
  const GAP = 14;
  const MARGIN = 12;

  let index = 0;
  let timer: ReturnType<typeof setInterval> | null = null;
  let animationFrame = 0;
  let anchorRect: DOMRect | null = null;
  let previousScreenshotKey = "";
  let loadedFullUrlSet = new Set<string>();
  let loadedThumbnailUrlSet = new Set<string>();
  let requestedUrls = new Set<string>();
  let showInfo = true;
  let infoTimer: ReturnType<typeof setTimeout> | null = null;

  const portal = (node: HTMLElement) => {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      },
    };
  };

  $: hasScreenshots = screenshots.length > 0;
  $: sourceImages = screenshots
    .filter((s) => s.url || s.thumbnail)
    .map((s) => ({
      fullUrl: s.url || s.thumbnail,
      thumbnailUrl: s.thumbnail || s.url,
    }));
  $: displayUrls = [
    fallbackImage,
    ...sourceImages
      .filter(
        (image) =>
          loadedThumbnailUrlSet.has(image.thumbnailUrl) ||
          loadedFullUrlSet.has(image.fullUrl),
      )
      .map((image) =>
        loadedFullUrlSet.has(image.fullUrl) ? image.fullUrl : image.thumbnailUrl,
      ),
  ];
  $: activeImageUrl =
    displayUrls.length > 0
      ? displayUrls[index % displayUrls.length]
      : fallbackImage;
  $: activeDisplayIndex =
    displayUrls.length > 0 ? index % displayUrls.length : 0;
  $: activeScreenshot = hasScreenshots
    ? screenshots.find((s) => (s.url || s.thumbnail) === activeImageUrl) ??
      screenshots[0]
    : null;
  $: fallbackImage = collectionElement.thumbnail
    ? `${convertFileSrc(collectionElement.thumbnail)}?v=${collectionElement.updatedAt}`
    : "/images/dummy_thumbnail.svg";
  $: playTimeText = formatPlayTime(collectionElement.totalPlayTimeSeconds);
  $: lastPlayedText = formatLastPlayed(collectionElement.lastPlayAt);

  $: screenshotKey = screenshots.map((s) => s.id).join("|");

  const restartInfoTimer = () => {
    showInfo = true;
    if (infoTimer) {
      clearTimeout(infoTimer);
    }
    infoTimer = setTimeout(() => {
      showInfo = false;
      infoTimer = null;
    }, 3000);
  };

  onMount(() => {
    restartInfoTimer();
  });

  const preloadImage = (url: string) => {
    return new Promise<void>((resolve) => {
      if (requestedUrls.has(url)) {
        resolve();
        return;
      }
      requestedUrls.add(url);
      const img = new Image();
      img.decoding = "async";
      img.onload = () => resolve();
      img.onerror = () => resolve();
      img.src = url;
    });
  };

  const preloadPreviewImage = (image: {
    fullUrl: string;
    thumbnailUrl: string;
  }) => {
    if (
      loadedThumbnailUrlSet.has(image.thumbnailUrl) ||
      loadedFullUrlSet.has(image.fullUrl)
    ) {
      return;
    }
    preloadImage(image.thumbnailUrl).then(() => {
      if (!loadedThumbnailUrlSet.has(image.thumbnailUrl)) {
        loadedThumbnailUrlSet = new Set([
          ...loadedThumbnailUrlSet,
          image.thumbnailUrl,
        ]);
      }
      if (image.fullUrl !== image.thumbnailUrl) {
        preloadImage(image.fullUrl).then(() => {
          if (!loadedFullUrlSet.has(image.fullUrl)) {
            loadedFullUrlSet = new Set([...loadedFullUrlSet, image.fullUrl]);
          }
        });
      } else if (!loadedFullUrlSet.has(image.fullUrl)) {
        loadedFullUrlSet = new Set([...loadedFullUrlSet, image.fullUrl]);
      }
    });
  };

  $: if (screenshotKey !== previousScreenshotKey) {
    previousScreenshotKey = screenshotKey;
    index = 0;
    loadedFullUrlSet = new Set<string>();
    loadedThumbnailUrlSet = new Set<string>();
    requestedUrls = new Set<string>();
    restartInfoTimer();
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  }

  $: sourceImages.forEach(preloadPreviewImage);

  $: if (displayUrls.length > 1 && !timer) {
    timer = setInterval(() => {
      index = (index + 1) % displayUrls.length;
    }, 1000);
  }

  $: if (displayUrls.length <= 1 && timer) {
    clearInterval(timer);
    timer = null;
  }

  $: style = (() => {
    if (!anchorRect) return "";
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const rightLeft = anchorRect.right + GAP;
    const leftLeft = anchorRect.left - WIDTH - GAP;
    const preferredLeft =
      rightLeft + WIDTH + MARGIN <= viewportWidth ? rightLeft : leftLeft;
    const left = Math.max(
      MARGIN,
      Math.min(preferredLeft, viewportWidth - WIDTH - MARGIN),
    );
    const top = Math.max(
      MARGIN,
      Math.min(anchorRect.top, viewportHeight - HEIGHT - MARGIN),
    );
    return `left: ${left}px; top: ${top}px; width: ${WIDTH}px; height: ${HEIGHT}px;`;
  })();

  const updateAnchorRect = () => {
    if (anchorElement?.isConnected) {
      anchorRect = anchorElement.getBoundingClientRect();
    } else {
      anchorRect = null;
    }
    animationFrame = requestAnimationFrame(updateAnchorRect);
  };

  $: if (anchorElement && !animationFrame) {
    updateAnchorRect();
  }

  onDestroy(() => {
    if (timer) {
      clearInterval(timer);
    }
    if (infoTimer) {
      clearTimeout(infoTimer);
    }
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
  });
</script>

{#if anchorRect}
  <div
    use:portal
    class="fixed z-[9999] pointer-events-none rounded-md overflow-hidden bg-bg-secondary shadow-2xl border border-white/10"
    {style}
  >
    <div class="absolute inset-0">
      <img
        src={fallbackImage}
        alt=""
        class="absolute inset-0 w-full h-full object-cover blur-md scale-110 opacity-70"
      />
      <div class="absolute inset-0 bg-white/18" />
      {#key activeImageUrl}
        <img
          src={activeImageUrl}
          alt=""
          class="relative w-full h-full object-contain"
        />
      {/key}
      <div
        class="absolute inset-x-0 bottom-0 h-[48%] transition-opacity duration-500"
        class:opacity-0={!showInfo}
        style="background: linear-gradient(to top, rgba(0, 0, 0, 0.82) 0%, rgba(0, 0, 0, 0.58) 28%, rgba(0, 0, 0, 0.26) 62%, transparent 100%);"
      />
    </div>

    <div class="relative h-full">
      <div
        class="absolute inset-x-4 bottom-12 min-w-0 transition-opacity duration-500"
        class:opacity-0={!showInfo}
      >
        <div class="truncate text-white text-lg font-bold drop-shadow">
          {collectionElement.gamename}
        </div>
      </div>

      <div class="absolute inset-x-4 bottom-4 flex items-end gap-3 min-w-0">
        <div
          class="min-w-0 flex-1 transition-opacity duration-500"
          class:opacity-0={!showInfo}
        >
          {#if collectionElement.brandname}
            <div class="truncate text-gray-200 text-xs drop-shadow">
              {collectionElement.brandname}
            </div>
          {:else}
            <div class="h-4" />
          {/if}
        </div>

        <div
          class="flex shrink-0 items-center gap-3 text-xs text-gray-200 transition-opacity duration-500"
          class:opacity-0={!showInfo}
        >
          {#if lastPlayedText}
            <div class="flex items-center gap-1">
              <div class="i-material-symbols-history-rounded w-4 h-4 shrink-0" />
              <span class="max-w-22 truncate">{lastPlayedText}</span>
            </div>
          {/if}
          {#if playTimeText}
            <div class="flex items-center gap-1">
              <div class="i-material-symbols-hourglass-outline-rounded w-4 h-4 shrink-0" />
              <span class="max-w-18 truncate">{playTimeText}</span>
            </div>
          {/if}
        </div>

        {#if displayUrls.length > 1}
          <div class="flex shrink-0 gap-1 pb-1">
            {#each displayUrls as _, i}
              <div
                class="h-1 rounded-full transition-colors duration-150"
                style="width: {Math.max(
                  12,
                  72 / displayUrls.length,
                )}px; background-color: rgba(255, 255, 255, {i ===
                activeDisplayIndex
                  ? 0.95
                  : 0.35});"
              />
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
