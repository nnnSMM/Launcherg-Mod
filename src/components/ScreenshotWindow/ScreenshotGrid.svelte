<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { Screenshot } from "@/lib/types";
    import { createEventDispatcher } from "svelte";

    export let screenshots: Screenshot[];

    let gridSize = 300;

    const dispatch = createEventDispatcher();

    const handleClick = (screenshot: Screenshot) => {
        dispatch("clickScreenshot", screenshot);
    };
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
        {#each screenshots as screenshot}
            <button
                class="relative aspect-video group overflow-hidden rounded-lg border border-border-primary bg-bg-secondary hover:border-accent-primary transition-all shadow-sm hover:shadow-md cursor-pointer"
                on:click={() => handleClick(screenshot)}
            >
                <img
                    src={convertFileSrc(screenshot.filename)}
                    alt={screenshot.filename}
                    class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                    loading="lazy"
                />
                <div
                    class="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors"
                />
                <div
                    class="absolute bottom-0 left-0 right-0 p-3 bg-gradient-to-t from-black/80 to-transparent opacity-0 group-hover:opacity-100 transition-opacity flex justify-between items-end"
                >
                    <div class="text-xs text-white/90 truncate font-mono">
                        {new Date(screenshot.createdAt).toLocaleString()}
                    </div>
                </div>
            </button>
        {/each}
    </div>
{/if}

<style>
    /* Scrollbar styles should be handled by parent container */
</style>
