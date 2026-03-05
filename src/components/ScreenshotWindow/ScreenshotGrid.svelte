<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { Screenshot } from "@/lib/types";
    import { createEventDispatcher } from "svelte";

    export let screenshots: Screenshot[] = [];
    export let selectionMode: boolean = false;
    export let selectedIds: Set<number> = new Set();
    export let scrollTop: number = 0;
    export let viewportHeight: number = 0;
    export let contentWidth: number = 0;

    const minTileWidth = 300;
    const gap = 16;
    const overscanRows = 3;
    const bottomPadding = 80;
    const aspectRatio = 16 / 9;

    const dispatch = createEventDispatcher();

    const handleClick = (screenshot: Screenshot) => {
        if (selectionMode) {
            dispatch("toggleSelection", screenshot.id);
        } else {
            dispatch("clickScreenshot", screenshot);
        }
    };

    type VisibleScreenshot = {
        screenshot: Screenshot;
        src: string;
        createdAtText: string;
        top: number;
        left: number;
        isSelected: boolean;
    };

    $: columnCount = Math.max(
        1,
        Math.floor((Math.max(contentWidth, minTileWidth) + gap) / (minTileWidth + gap)),
    );
    $: tileWidth = Math.floor(
        (Math.max(contentWidth, minTileWidth) - gap * (columnCount - 1)) /
            columnCount,
    );
    $: tileHeight = Math.floor(tileWidth / aspectRatio);
    $: rowHeight = tileHeight + gap;
    $: totalRows = Math.ceil(screenshots.length / columnCount);
    $: virtualHeight = totalRows > 0
        ? totalRows * rowHeight - gap + bottomPadding
        : 0;
    $: maxScrollTop = Math.max(
        0,
        (totalRows > 0 ? totalRows * rowHeight - gap : 0) -
            Math.max(viewportHeight, 0),
    );
    $: clampedScrollTop = Math.min(Math.max(scrollTop, 0), maxScrollTop);
    $: startRow = viewportHeight > 0
        ? Math.max(0, Math.floor(clampedScrollTop / rowHeight) - overscanRows)
        : 0;
    $: endRow = viewportHeight > 0
        ? Math.min(
            totalRows - 1,
            Math.ceil((clampedScrollTop + viewportHeight) / rowHeight) +
                overscanRows,
        )
        : Math.max(0, totalRows - 1);
    $: startIndex = startRow * columnCount;
    $: endIndex = Math.min(
        screenshots.length,
        (endRow + 1) * columnCount,
    );
    $: visibleScreenshots = screenshots
        .slice(startIndex, endIndex)
        .map((screenshot, localIndex): VisibleScreenshot => {
            const index = startIndex + localIndex;
            const row = Math.floor(index / columnCount);
            const col = index % columnCount;
            return {
                screenshot,
                src: convertFileSrc(
                    screenshot.thumbnailFilename ?? screenshot.filename,
                ),
                createdAtText: new Date(screenshot.createdAt).toLocaleString(),
                top: row * rowHeight,
                left: col * (tileWidth + gap),
                isSelected: selectedIds.has(screenshot.id),
            };
        });
</script>

{#if screenshots.length === 0}
    <div
        class="flex items-center justify-center h-full text-text-secondary select-none"
    >
        <div class="text-center">
            <div
                class="i-material-symbols-image-not-supported text-6xl mb-4 opacity-50 mx-auto"
            />
            <p>No screenshots found</p>
        </div>
    </div>
{:else}
    <div class="relative" style="height: {virtualHeight}px;">
        {#each visibleScreenshots as item (item.screenshot.id)}
            {@const screenshot = item.screenshot}
            <button
                class="absolute group overflow-hidden bg-bg-secondary transition-all shadow-sm hover:shadow-md cursor-pointer {selectionMode &&
                item.isSelected
                    ? 'border-accent-primary border-2 scale-95 opacity-80'
                    : ''}"
                style="left: {item.left}px; top: {item.top}px; width: {tileWidth}px; height: {tileHeight}px;"
                on:click={() => handleClick(screenshot)}
            >
                {#if selectionMode}
                    <div
                        class="absolute top-2 left-2 z-10 transition-transform hover:scale-110"
                    >
                        <div
                            class="w-6 h-6 rounded-full border-2 flex items-center justify-center transition-colors {item.isSelected
                                ? 'bg-accent-primary border-accent-primary'
                                : 'bg-black/50 border-white/70'}"
                        >
                            {#if item.isSelected}
                                <div
                                    class="i-material-symbols-check text-white text-sm"
                                />
                            {/if}
                        </div>
                    </div>
                {/if}
                <img
                    src={item.src}
                    alt={screenshot.filename}
                    class="absolute inset-0 w-full h-full object-cover transition-transform duration-300 {selectionMode
                        ? ''
                        : 'group-hover:scale-105'}"
                    loading="lazy"
                    decoding="async"
                />
                <div
                    class="absolute inset-0 bg-black/0 {selectionMode
                        ? ''
                        : 'group-hover:bg-black/20'} transition-colors"
                />
                <div
                    class="absolute bottom-0 left-0 right-0 p-3 bg-gradient-to-t from-black/80 to-transparent opacity-0 {selectionMode
                        ? ''
                        : 'group-hover:opacity-100'} transition-opacity flex justify-between items-end"
                >
                    <div class="text-xs text-white/90 truncate font-mono">
                        {item.createdAtText}
                    </div>
                </div>
            </button>
        {/each}
    </div>
{/if}

<style>
    /* Scrollbar styles should be handled by parent container */
</style>
