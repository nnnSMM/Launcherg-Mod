<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { Screenshot } from "@/lib/types";
    import { createEventDispatcher } from "svelte";

    export let screenshots: Screenshot[] = [];
    export let selectionMode: boolean = false;
    export let selectedIds: Set<number> = new Set();

    let gridSize = 300;

    const dispatch = createEventDispatcher();

    const handleClick = (screenshot: Screenshot) => {
        if (selectionMode) {
            dispatch("toggleSelection", screenshot.id);
        } else {
            dispatch("clickScreenshot", screenshot);
        }
    };

    type DisplayScreenshot = {
        screenshot: Screenshot;
        src: string;
        createdAtText: string;
    };

    $: displayScreenshots = screenshots.map((screenshot): DisplayScreenshot => ({
        screenshot,
        src: convertFileSrc(screenshot.filename),
        createdAtText: new Date(screenshot.createdAt).toLocaleString(),
    }));
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
    <div
        class="grid gap-4 pb-20"
        style="grid-template-columns: repeat(auto-fill, minmax({gridSize}px, 1fr));"
    >
        {#each displayScreenshots as item (item.screenshot.id)}
            {@const screenshot = item.screenshot}
            {@const isSelected = selectedIds.has(screenshot.id)}
            <button
                class="relative aspect-video group overflow-hidden bg-bg-secondary transition-all shadow-sm hover:shadow-md cursor-pointer {selectionMode &&
                isSelected
                    ? 'border-accent-primary border-2 scale-95 opacity-80'
                    : ''}"
                on:click={() => handleClick(screenshot)}
            >
                {#if selectionMode}
                    <div
                        class="absolute top-2 left-2 z-10 transition-transform hover:scale-110"
                    >
                        <div
                            class="w-6 h-6 rounded-full border-2 flex items-center justify-center transition-colors {isSelected
                                ? 'bg-accent-primary border-accent-primary'
                                : 'bg-black/50 border-white/70'}"
                        >
                            {#if isSelected}
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
                    class="w-full h-full object-cover transition-transform duration-300 {selectionMode
                        ? ''
                        : 'group-hover:scale-105'}"
                    loading="lazy"
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
