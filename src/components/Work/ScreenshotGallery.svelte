<script lang="ts">
    import { onMount } from "svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { Screenshot } from "@/lib/types";
    import {
        commandGetGameScreenshots,
        commandImportScreenshot,
        commandGetAppSetting,
        commandSetAppSetting,
        commandUpdateScreenshotsOrder,
    } from "@/lib/command";
    import ScreenshotViewer from "./ScreenshotViewer.svelte";

    export let gameId: number;

    let screenshots: Screenshot[] = [];
    let viewerOpen = false;
    let viewerInitialIndex = 0;
    let gridSize = 200;
    let saveTimeout: number | null = null;

    // Drag and drop state
    let draggedIndex: number | null = null;
    let dragOverIndex: number | null = null;
    let longPressTimer: number | null = null;
    let isDragging = false;
    let preventClick = false;

    onMount(async () => {
        await loadScreenshots();
        await loadGridSize();

        //  Window-level pointermove for drag detection
        window.addEventListener("pointermove", handleWindowPointerMove);
        return () => {
            window.removeEventListener("pointermove", handleWindowPointerMove);
        };
    });

    const loadScreenshots = async () => {
        try {
            screenshots = await commandGetGameScreenshots(gameId);
        } catch (e) {
            console.error("Failed to load screenshots", e);
        }
    };

    const loadGridSize = async () => {
        try {
            const size = await commandGetAppSetting("screenshot_grid_size");
            if (size) {
                const parsed = parseInt(size);
                if (!isNaN(parsed)) {
                    gridSize = parsed;
                }
            }
        } catch (e) {
            console.error("Failed to load grid size", e);
        }
    };

    const saveGridSize = (size: number) => {
        if (saveTimeout !== null) {
            clearTimeout(saveTimeout);
        }
        saveTimeout = window.setTimeout(async () => {
            try {
                await commandSetAppSetting(
                    "screenshot_grid_size",
                    size.toString(),
                );
            } catch (e) {
                console.error("Failed to save grid size", e);
            }
        }, 500);
    };

    const handleImport = async () => {
        try {
            const selected = await open({
                multiple: true,
                filters: [
                    {
                        name: "Images",
                        extensions: ["png", "jpg", "jpeg", "webp"],
                    },
                ],
            });

            if (!selected) return;

            let paths: string[] = [];
            if (Array.isArray(selected)) {
                paths = selected.map((s) =>
                    typeof s === "string" ? s : (s as any).path,
                );
            } else if (typeof selected === "string") {
                paths = [selected];
            } else if (selected && typeof selected === "object") {
                if ("path" in selected) {
                    paths = [(selected as any).path];
                }
            }

            for (const path of paths) {
                await commandImportScreenshot(gameId, path);
            }

            await loadScreenshots();
        } catch (e) {
            console.error("Failed to import screenshot", e);
        }
    };

    const openViewer = (index: number) => {
        if (preventClick) {
            preventClick = false;
            return;
        }
        if (!isDragging) {
            viewerInitialIndex = index;
            viewerOpen = true;
        }
    };

    const closeViewer = () => {
        viewerOpen = false;
        loadScreenshots();
    };

    // Long press and drag handlers
    const handlePointerDown = (e: PointerEvent, index: number) => {
        longPressTimer = window.setTimeout(() => {
            isDragging = true;
            draggedIndex = index;
        }, 200);
    };

    const handleWindowPointerMove = (e: PointerEvent) => {
        if (isDragging && draggedIndex !== null) {
            const elements = document.elementsFromPoint(e.clientX, e.clientY);

            for (const el of elements) {
                const button = el.closest("[data-screenshot-index]");
                if (button) {
                    const index = parseInt(
                        button.getAttribute("data-screenshot-index") || "-1",
                    );
                    if (index !== -1 && index !== dragOverIndex) {
                        dragOverIndex = index;
                    }
                    break;
                }
            }
        }
    };

    const handlePointerUp = async (e: PointerEvent) => {
        if (longPressTimer !== null) {
            clearTimeout(longPressTimer);
            longPressTimer = null;
        }

        if (isDragging) {
            preventClick = true;

            if (
                draggedIndex !== null &&
                dragOverIndex !== null &&
                draggedIndex !== dragOverIndex
            ) {
                const newScreenshots = [...screenshots];
                const [removed] = newScreenshots.splice(draggedIndex, 1);
                newScreenshots.splice(dragOverIndex, 0, removed);

                const updates = newScreenshots.map((screenshot, index) => ({
                    id: screenshot.id,
                    orderIndex: index,
                }));

                try {
                    await commandUpdateScreenshotsOrder(updates);
                    screenshots = newScreenshots;
                } catch (e) {
                    console.error(
                        "[Drag] Failed to update screenshot order",
                        e,
                    );
                    await loadScreenshots();
                }
            }
        }

        isDragging = false;
        draggedIndex = null;
        dragOverIndex = null;
    };

    const handlePointerCancel = (e: PointerEvent) => {
        if (longPressTimer !== null) {
            clearTimeout(longPressTimer);
            longPressTimer = null;
        }

        isDragging = false;
        draggedIndex = null;
        dragOverIndex = null;
    };

    $: {
        if (gridSize) {
            saveGridSize(gridSize);
        }
    }
</script>

<svelte:window on:pointerup={handlePointerUp} />

<div class="w-full p-4">
    <div class="flex justify-between items-center mb-4">
        <h2 class="text-xl font-bold text-text-primary">Screenshots</h2>
        <div class="flex items-center gap-4">
            <div class="flex items-center gap-2">
                <label
                    for="grid-size"
                    class="text-sm text-text-secondary whitespace-nowrap"
                >
                    Grid Size:
                </label>
                <input
                    id="grid-size"
                    type="range"
                    min="150"
                    max="400"
                    step="50"
                    bind:value={gridSize}
                    class="w-32"
                />
                <span class="text-sm text-text-secondary w-12"
                    >{gridSize}px</span
                >
            </div>
            <button
                on:click={handleImport}
                class="flex items-center gap-2 px-4 py-2 bg-accent-accent text-white rounded hover:bg-accent-accent/80 transition-colors"
            >
                <div class="i-material-symbols-add text-xl" />
                Import
            </button>
        </div>
    </div>

    {#if screenshots.length === 0}
        <div class="text-center text-text-secondary py-8 text-sm">
            まだスクリーンショットを撮影していません。Windows+Shift+SやWindows+PrintScreenを押して撮影してください。
        </div>
    {:else}
        <div
            class="grid gap-4"
            style="grid-template-columns: repeat(auto-fill, minmax({gridSize}px, 1fr));"
        >
            {#each screenshots as screenshot, i}
                <button
                    data-screenshot-index={i}
                    class="relative aspect-video group overflow-hidden rounded-lg border transition-all {dragOverIndex ===
                        i && isDragging
                        ? 'border-accent-accent border-2 scale-105'
                        : 'border-border-primary hover:border-accent-primary'} {draggedIndex ===
                        i && isDragging
                        ? 'opacity-50'
                        : ''}"
                    on:click={() => openViewer(i)}
                    on:pointerdown={(e) => handlePointerDown(e, i)}
                    on:pointercancel={handlePointerCancel}
                    style="touch-action: none; cursor: {isDragging
                        ? 'grabbing'
                        : 'pointer'};"
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
                    {#if isDragging && draggedIndex === i}
                        <div
                            class="absolute inset-0 flex items-center justify-center bg-black/50"
                        >
                            <div
                                class="i-material-symbols-drag-indicator text-4xl text-white"
                            />
                        </div>
                    {/if}
                </button>
            {/each}
        </div>
    {/if}
</div>

{#if viewerOpen}
    <ScreenshotViewer
        bind:screenshots
        initialIndex={viewerInitialIndex}
        on:close={closeViewer}
    />
{/if}
