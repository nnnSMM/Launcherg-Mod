<script lang="ts">
    import { onMount, tick } from "svelte";
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
    let currentWindowHandle: Awaited<
        ReturnType<typeof import("@tauri-apps/api/window").getCurrentWindow>
    > | null = null;
    let hasEmittedInitialReady = false;
    let viewerImageReady = false;
    let lastReadyScreenshotId: number | null = null;
    let readyFallbackTimer: ReturnType<typeof setTimeout> | null = null;
    let isBootstrapping = true;

    // Initial args from window creation
    type WindowArgs = {
        game_id: number | null;
        initial_screenshot_id: number | null;
        initial_screenshot: Screenshot | null;
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

        // Fast path: show clicked screenshot immediately.
        if (args.initial_screenshot) {
            viewerScreenshots = [args.initial_screenshot];
            currentIndex = 0;
            viewMode = "viewer";
        }

        // Resolve full list once data is loaded.
        if (args.initial_screenshot_id && screenshots.length > 0) {
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
        } else if (args.initial_screenshot && screenshots.length > 0) {
            const currentFiltered = selectedGameId
                ? screenshots.filter((s) => s.gameId === selectedGameId)
                : screenshots;
            const idx = currentFiltered.findIndex(
                (s) => s.id === args.initial_screenshot!.id,
            );
            if (idx >= 0) {
                viewerScreenshots = currentFiltered;
                currentIndex = idx;
                viewMode = "viewer";
            }
        }
    };


    const emitInitialReady = async () => {
        if (hasEmittedInitialReady || !currentWindowHandle || isBootstrapping) return;
        await tick();
        await currentWindowHandle.emit("screenshot-window-ready", { ready: true });
        hasEmittedInitialReady = true;
    };

    onMount(async () => {
        // Use getCurrentWindow().listen for window-specific events
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const currentWindow = getCurrentWindow();
        currentWindowHandle = currentWindow;
        readyFallbackTimer = setTimeout(() => {
            if (viewMode !== "viewer") {
                void emitInitialReady();
            }
        }, 2000);

        const unlisten = await currentWindow.listen<WindowArgs>(
            "screenshot-window-args",
            async (event) => {
                if (event.payload.initial_screenshot) {
                    handleArgs(event.payload, []);
                    await tick();
                    await currentWindow.emit("screenshot-window-args-applied", {
                        applied: true,
                    });
                }

                const freshData = await fetchData();
                handleArgs(event.payload, freshData);
                await tick();
                await currentWindow.emit("screenshot-window-args-applied", {
                    applied: true,
                });
            },
        );

        // Check for initial args set by backend script injection
        const w = window as any;
        const initialArgs =
            (w.__INITIAL_SCREENSHOT_ARGS__ as WindowArgs | null) ?? null;
        if (initialArgs) {
            handleArgs(initialArgs, []);
            w.__INITIAL_SCREENSHOT_ARGS__ = null;
            if (initialArgs.initial_screenshot) {
                isBootstrapping = false;
            }
        }

        const freshScreenshots = await fetchData();
        if (initialArgs) {
            handleArgs(initialArgs, freshScreenshots);
        }

        isBootstrapping = false;

        return () => {
            if (readyFallbackTimer) {
                clearTimeout(readyFallbackTimer);
                readyFallbackTimer = null;
            }
            unlisten();
        };
    });

    $: {
        const currentId = currentScreenshot?.id ?? null;
        if (currentId !== lastReadyScreenshotId) {
            lastReadyScreenshotId = currentId;
            viewerImageReady = false;
        }
    }

    $: if (!hasEmittedInitialReady && currentWindowHandle && !isBootstrapping) {
        if (viewMode !== "viewer") {
            void emitInitialReady();
        } else if (currentScreenshot && viewerImageReady) {
            void emitInitialReady();
        }
    }

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

    const handleViewerImageReady = () => {
        viewerImageReady = true;
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

    // Selection Mode
    let selectionMode = false;
    let selectedIds: Set<number> = new Set();

    const toggleSelectionMode = () => {
        selectionMode = !selectionMode;
        if (!selectionMode) {
            selectedIds.clear();
            selectedIds = selectedIds;
        }
    };

    const handleToggleSelection = (id: number) => {
        if (selectedIds.has(id)) {
            selectedIds.delete(id);
        } else {
            selectedIds.add(id);
        }
        selectedIds = selectedIds;
    };

    // Delete functionality
    const confirmDelete = () => {
        showDeleteConfirm = true;
    };

    const cancelDelete = () => {
        showDeleteConfirm = false;
    };

    const handleDelete = async () => {
        showDeleteConfirm = false;

        if (viewMode === "grid" && selectionMode) {
            try {
                for (const id of selectedIds) {
                    await commandDeleteScreenshot(id);
                }
                const deletedIdsSet = new Set(selectedIds);
                allScreenshots = allScreenshots.filter(
                    (s) => !deletedIdsSet.has(s.id),
                );
                selectedIds.clear();
                selectedIds = selectedIds;
                selectionMode = false;
            } catch (e) {
                console.error("Failed to batch delete screenshots", e);
            }
        } else if (currentScreenshot && viewMode === "viewer") {
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
    {#if isBootstrapping}
        <div class="flex-1 min-h-0 bg-black" />
    {:else if viewMode === "viewer" && currentScreenshot}
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
                on:ready={handleViewerImageReady}
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

            <div class="flex items-center gap-2">
                {#if selectionMode}
                    <button
                        class="px-3 py-1.5 text-sm rounded bg-bg-tertiary text-text-primary hover:bg-bg-button transition-colors"
                        on:click={toggleSelectionMode}
                    >
                        キャンセル
                    </button>
                    <button
                        class="px-3 py-1.5 text-sm rounded transition-colors {selectedIds.size >
                        0
                            ? 'bg-red-500 text-white hover:bg-red-600'
                            : 'bg-bg-tertiary text-text-secondary cursor-not-allowed opacity-50'}"
                        on:click={confirmDelete}
                        disabled={selectedIds.size === 0}
                    >
                        削除 ({selectedIds.size}件)
                    </button>
                {:else}
                    <button
                        class="px-3 py-1.5 text-sm rounded bg-bg-tertiary text-text-primary hover:bg-bg-button transition-colors flex items-center gap-1"
                        on:click={toggleSelectionMode}
                        title="一括編集"
                    >
                        <span class="i-material-symbols-checklist text-lg" />
                        選択
                    </button>
                {/if}
            </div>
        </div>

        <!-- Content -->
        <div class="flex-1 min-h-0 relative">
            <div class="h-full w-full overflow-y-auto p-4 custom-scrollbar">
                <ScreenshotGrid
                    screenshots={filteredScreenshots}
                    {selectionMode}
                    {selectedIds}
                    on:reload={fetchData}
                    on:clickScreenshot={(e) => enterViewer(e.detail)}
                    on:toggleSelection={(e) => handleToggleSelection(e.detail)}
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
                    {#if selectionMode}
                        選択した {selectedIds.size} 件のスクリーンショットを削除してもよろしいですか？この操作は取り消せません。
                    {:else}
                        このスクリーンショットを削除してもよろしいですか？この操作は取り消せません。
                    {/if}
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
