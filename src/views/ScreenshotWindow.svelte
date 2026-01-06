<script lang="ts">
    import { onMount } from "svelte";
    import type { Screenshot, CollectionElement } from "@/lib/types";
    import {
        commandGetAllScreenshots,
        commandGetAllElements,
        commandDeleteScreenshot,
    } from "@/lib/command";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { stat } from "@tauri-apps/plugin-fs";
    import ScreenshotGrid from "@/components/ScreenshotWindow/ScreenshotGrid.svelte";
    import GameSelector from "@/components/ScreenshotWindow/GameSelector.svelte";
    import ZoomableImage from "@/components/UI/ZoomableImage.svelte";

    let allScreenshots: Screenshot[] = [];
    let allGames: CollectionElement[] = [];

    let viewMode: "grid" | "viewer" = "grid";
    let selectedGameId: number | null = null;

    // For viewer
    let viewerScreenshots: Screenshot[] = [];
    let currentIndex: number = 0;
    let currentScreenshot: Screenshot | null = null;
    let currentFileSize: number | null = null;
    let isZoomed = false;
    let showDeleteConfirm = false;

    // Initial args from window creation
    type WindowArgs = {
        game_id: number | null;
        initial_screenshot_id: number | null;
    };

    const fetchData = async (): Promise<Screenshot[]> => {
        try {
            const [s, g] = await Promise.all([
                commandGetAllScreenshots(),
                commandGetAllElements(),
            ]);
            allScreenshots = s;
            allGames = g.sort((a, b) => a.gamename.localeCompare(b.gamename));
            return s;
        } catch (e) {
            console.error(e);
            return [];
        }
    };

    // Filter games to only show those that have screenshots
    $: gameIdsWithScreenshots = new Set(allScreenshots.map((s) => s.gameId));
    $: gamesWithScreenshots = allGames.filter((g) =>
        gameIdsWithScreenshots.has(g.id),
    );

    const handleArgs = (args: WindowArgs, screenshots: Screenshot[]) => {
        // Always set the game filter if provided
        if (args.game_id !== undefined && args.game_id !== null) {
            selectedGameId = args.game_id;
        }

        // If initial_screenshot_id is provided, switch to viewer mode
        if (args.initial_screenshot_id) {
            const target = screenshots.find(
                (s) => s.id === args.initial_screenshot_id,
            );

            if (target) {
                const currentFiltered = selectedGameId
                    ? screenshots.filter((s) => s.gameId === selectedGameId)
                    : screenshots;

                viewerScreenshots = currentFiltered;
                const idx = viewerScreenshots.findIndex(
                    (s) => s.id === args.initial_screenshot_id,
                );
                currentIndex = idx >= 0 ? idx : 0;
                viewMode = "viewer";
            }
        }
    };

    onMount(async () => {
        const freshScreenshots = await fetchData();

        // Check for initial args set by backend script injection
        const w = window as any;
        if (w.__INITIAL_SCREENSHOT_ARGS__) {
            handleArgs(w.__INITIAL_SCREENSHOT_ARGS__, freshScreenshots);
            w.__INITIAL_SCREENSHOT_ARGS__ = null;
        }

        // Use getCurrentWindow().listen for window-specific events
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const currentWindow = getCurrentWindow();

        const unlisten = await currentWindow.listen<WindowArgs>(
            "screenshot-window-args",
            async (event) => {
                const freshData = await fetchData();
                handleArgs(event.payload, freshData);
            },
        );

        return () => {
            unlisten();
        };
    });

    $: filteredScreenshots = selectedGameId
        ? allScreenshots.filter((s) => s.gameId === selectedGameId)
        : allScreenshots;

    // Update currentScreenshot when index or screenshots change
    $: {
        if (viewerScreenshots.length > 0) {
            if (currentIndex >= viewerScreenshots.length)
                currentIndex = viewerScreenshots.length - 1;
            if (currentIndex < 0) currentIndex = 0;
            currentScreenshot = viewerScreenshots[currentIndex];
            // Fetch file size
            if (currentScreenshot) {
                console.log(
                    "[ScreenshotWindow] Getting file size for:",
                    currentScreenshot.filename,
                );
                stat(currentScreenshot.filename)
                    .then((s) => {
                        console.log("[ScreenshotWindow] File size:", s.size);
                        currentFileSize = s.size;
                    })
                    .catch((e) => {
                        console.error(
                            "[ScreenshotWindow] Failed to get file size:",
                            e,
                        );
                        currentFileSize = null;
                    });
            }
        } else if (viewMode === "viewer") {
            backToGrid();
        }
    }

    const enterViewer = (targetScreenshot: Screenshot) => {
        viewerScreenshots = filteredScreenshots;
        const idx = viewerScreenshots.findIndex(
            (s) => s.id === targetScreenshot.id,
        );
        currentIndex = idx >= 0 ? idx : 0;
        viewMode = "viewer";
    };

    const backToGrid = () => {
        viewMode = "grid";
        isZoomed = false;
    };

    const handleGameSelect = (id: number | null) => {
        selectedGameId = id;
        viewMode = "grid";
    };

    $: currentGame = currentScreenshot
        ? allGames.find((g) => g.id === currentScreenshot!.gameId)
        : null;

    $: currentGameName = currentGame?.gamename || null;

    $: currentGameIcon = currentGame?.icon
        ? convertFileSrc(currentGame.icon)
        : null;

    // Viewer navigation
    const next = () => {
        currentIndex = (currentIndex + 1) % viewerScreenshots.length;
    };

    const prev = () => {
        currentIndex =
            (currentIndex - 1 + viewerScreenshots.length) %
            viewerScreenshots.length;
    };

    const handleKeydown = (e: KeyboardEvent) => {
        if (viewMode !== "viewer") return;
        if (e.key === "Escape") backToGrid();
        if (e.key === "ArrowRight") next();
        if (e.key === "ArrowLeft") prev();
    };

    // Delete functionality
    const confirmDelete = () => {
        showDeleteConfirm = true;
    };

    const cancelDelete = () => {
        showDeleteConfirm = false;
    };

    const handleDelete = async () => {
        if (!currentScreenshot) return;
        showDeleteConfirm = false;
        try {
            await commandDeleteScreenshot(currentScreenshot.id);
            viewerScreenshots = viewerScreenshots.filter(
                (s) => s.id !== currentScreenshot!.id,
            );
            allScreenshots = allScreenshots.filter(
                (s) => s.id !== currentScreenshot!.id,
            );
            if (viewerScreenshots.length === 0) {
                backToGrid();
            } else if (currentIndex >= viewerScreenshots.length) {
                currentIndex = viewerScreenshots.length - 1;
            }
        } catch (e) {
            console.error("Failed to delete screenshot", e);
        }
    };

    // Format file size
    const formatFileSize = (bytes: number): string => {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    };
</script>

<svelte:window on:keydown={handleKeydown} />

<div
    class="h-screen w-screen bg-bg-primary text-text-primary flex flex-col overflow-hidden"
>
    {#if viewMode === "viewer" && currentScreenshot}
        <!-- Viewer Mode -->
        <!-- Header -->
        <div
            class="h-12 flex items-center px-4 shrink-0 bg-[#1a1a1c] border-b border-[#333] justify-between z-10"
        >
            <button
                class="flex items-center space-x-1 px-3 py-1.5 rounded bg-[#333] text-white hover:bg-[#444] transition-colors"
                on:click={backToGrid}
            >
                <span class="i-material-symbols-chevron-left text-lg" />
                <span>戻る</span>
            </button>

            <div
                class="flex items-center space-x-2 absolute left-1/2 transform -translate-x-1/2"
            >
                {#if currentGameIcon}
                    <img
                        src={currentGameIcon}
                        alt=""
                        class="w-5 h-5 rounded object-cover"
                    />
                {:else}
                    <span
                        class="i-material-symbols-image text-accent-primary"
                    />
                {/if}
                <span class="text-sm text-[#c0c0c0]"
                    >{currentGameName || "すべてのメディア"}</span
                >
            </div>

            <div></div>
        </div>

        <!-- Image Content -->
        <div
            class="flex-1 min-h-0 relative bg-black flex items-center justify-center"
            on:click|self={backToGrid}
            role="button"
            tabindex="0"
            on:keydown={(e) => e.key === "Escape" && backToGrid()}
        >
            <!-- Navigation arrows -->
            <button
                class="absolute left-0 top-0 h-full px-6 flex items-center justify-center text-white bg-black/0 hover:bg-black/30 transition-all opacity-0 hover:opacity-100 z-40"
                class:!hidden={isZoomed || viewerScreenshots.length <= 1}
                on:click={prev}
            >
                <div
                    class="i-material-symbols-chevron-left text-6xl drop-shadow-lg"
                />
            </button>

            <button
                class="absolute right-0 top-0 h-full px-6 flex items-center justify-center text-white bg-black/0 hover:bg-black/30 transition-all opacity-0 hover:opacity-100 z-40"
                class:!hidden={isZoomed || viewerScreenshots.length <= 1}
                on:click={next}
            >
                <div
                    class="i-material-symbols-chevron-right text-6xl drop-shadow-lg"
                />
            </button>

            <ZoomableImage
                bind:isZoomed
                src={convertFileSrc(currentScreenshot.filename)}
                alt={currentScreenshot.filename}
                class="max-w-full max-h-full"
            />
        </div>

        <!-- Bottom Bar -->
        <div
            class="h-12 flex items-center px-4 shrink-0 bg-[#1a1a1c] border-t border-[#333] justify-between z-10"
            class:opacity-0={isZoomed}
            class:pointer-events-none={isZoomed}
        >
            <div class="text-sm text-[#8b8b8b]">
                撮影: {new Date(currentScreenshot.createdAt).toLocaleString(
                    "ja-JP",
                )}
                {#if currentFileSize !== null}
                    <span class="mx-2">•</span>
                    {formatFileSize(currentFileSize)}
                {/if}
            </div>

            <div class="flex items-center space-x-2">
                <button
                    class="flex items-center space-x-1 px-3 py-1.5 rounded bg-[#333] text-white hover:bg-[#444] transition-colors"
                    on:click={confirmDelete}
                    title="削除"
                >
                    <span class="i-material-symbols-delete text-lg" />
                </button>
            </div>
        </div>
    {:else}
        <!-- Grid Mode -->
        <!-- Header -->
        <div
            class="h-14 border-b border-border-primary flex items-center px-4 shrink-0 bg-bg-secondary justify-between z-10"
        >
            <div class="flex items-center space-x-3">
                <span class="text-accent-primary font-bold"
                    >スクリーンショット</span
                >

                <div class="h-6 w-px bg-border-primary"></div>

                <GameSelector
                    games={gamesWithScreenshots}
                    {selectedGameId}
                    on:select={(e) => handleGameSelect(e.detail)}
                />
            </div>
        </div>

        <!-- Content -->
        <div class="flex-1 min-h-0 relative">
            <div class="h-full w-full overflow-y-auto p-4 custom-scrollbar">
                <ScreenshotGrid
                    screenshots={filteredScreenshots}
                    on:reload={fetchData}
                    on:clickScreenshot={(e) => enterViewer(e.detail)}
                />
            </div>
        </div>
    {/if}

    <!-- Delete Confirmation Dialog -->
    {#if showDeleteConfirm}
        <div
            class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 backdrop-blur-sm"
            on:click|self={cancelDelete}
            on:keydown={(e) => e.key === "Escape" && cancelDelete()}
            role="dialog"
            aria-modal="true"
            tabindex="-1"
        >
            <div
                class="bg-bg-secondary border border-border-primary rounded-lg shadow-2xl p-6 max-w-md w-full mx-4"
            >
                <h3 class="text-lg font-bold text-text-primary mb-2">
                    削除の確認
                </h3>
                <p class="text-text-secondary mb-6">
                    このスクリーンショットを削除してもよろしいですか？この操作は取り消せません。
                </p>
                <div class="flex justify-end gap-3">
                    <button
                        class="px-4 py-2 rounded bg-bg-tertiary text-text-primary hover:bg-bg-button transition-colors"
                        on:click={cancelDelete}
                    >
                        キャンセル
                    </button>
                    <button
                        class="px-4 py-2 rounded bg-red-500 text-white hover:bg-red-600 transition-colors font-medium"
                        on:click={handleDelete}
                    >
                        削除
                    </button>
                </div>
            </div>
        </div>
    {/if}
</div>
