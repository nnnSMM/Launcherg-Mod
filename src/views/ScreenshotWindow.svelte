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
    import TitleBar from "@/components/TitleBar.svelte";
    import emblaCarouselSvelte from "embla-carousel-svelte";
    import type { EmblaCarouselType, EmblaOptionsType } from "embla-carousel";

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
    let isFullscreenViewer = false;
    let showFullscreenCursor = false;
    let showFullscreenChrome = false;
    let showFullscreenFilmstrip = false;
    let fullscreenUiTimer: ReturnType<typeof setTimeout> | null = null;
    let fullscreenFilmstripApi: EmblaCarouselType | null = null;
    const fullscreenFilmstripOptions: EmblaOptionsType = {
        align: "start",
        containScroll: "trimSnaps",
        dragFree: true,
    };
    let currentWindowHandle: Awaited<
        ReturnType<typeof import("@tauri-apps/api/window").getCurrentWindow>
    > | null = null;
    let hasEmittedInitialReady = false;
    let viewerImageReady = false;
    let lastReadyScreenshotId: number | null = null;
    let readyFallbackTimer: ReturnType<typeof setTimeout> | null = null;
    let isBootstrapping = true;
    let fileSizeRequestId = 0;
    const fileSizeCache = new Map<number, number | null>();
    let gridScrollContainer: HTMLDivElement | null = null;
    let gridScrollTop = 0;
    let gridViewportHeight = 0;
    let gridContentWidth = 0;
    let gridScrollRaf = 0;

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

    const observeGridContainer = (node: HTMLDivElement) => {
        gridScrollContainer = node;

        const px = (value: string): number => {
            const parsed = Number.parseFloat(value);
            return Number.isFinite(parsed) ? parsed : 0;
        };

        const updateMeasurements = () => {
            const style = getComputedStyle(node);
            const horizontalPadding =
                px(style.paddingLeft) + px(style.paddingRight);
            const verticalPadding = px(style.paddingTop) + px(style.paddingBottom);

            gridViewportHeight = Math.max(0, node.clientHeight - verticalPadding);
            gridContentWidth = Math.max(0, node.clientWidth - horizontalPadding);
        };

        const flushScroll = () => {
            gridScrollRaf = 0;
            gridScrollTop = node.scrollTop;
        };

        const onScroll = () => {
            if (gridScrollRaf !== 0) return;
            gridScrollRaf = requestAnimationFrame(flushScroll);
        };

        node.addEventListener("scroll", onScroll, { passive: true });
        const resizeObserver = new ResizeObserver(() => {
            updateMeasurements();
        });
        resizeObserver.observe(node);

        updateMeasurements();
        gridScrollTop = node.scrollTop;

        return {
            destroy() {
                node.removeEventListener("scroll", onScroll);
                resizeObserver.disconnect();
                if (gridScrollRaf !== 0) {
                    cancelAnimationFrame(gridScrollRaf);
                    gridScrollRaf = 0;
                }
                if (gridScrollContainer === node) {
                    gridScrollContainer = null;
                }
            },
        };
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
            if (fullscreenUiTimer) {
                clearTimeout(fullscreenUiTimer);
                fullscreenUiTimer = null;
            }
            if (isFullscreenViewer) {
                void currentWindow.setFullscreen(false);
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

    $: {
        const validIds = new Set(allScreenshots.map((s) => s.id));
        for (const id of fileSizeCache.keys()) {
            if (!validIds.has(id)) {
                fileSizeCache.delete(id);
            }
        }
    }

    // Update currentScreenshot when index or screenshots change
    $: {
        if (viewerScreenshots.length > 0) {
            if (currentIndex >= viewerScreenshots.length)
                currentIndex = viewerScreenshots.length - 1;
            if (currentIndex < 0) currentIndex = 0;
            currentScreenshot = viewerScreenshots[currentIndex];
        } else if (viewMode === "viewer") {
            backToGrid();
        }
    }

    $: if (currentScreenshot) {
        const cached = fileSizeCache.get(currentScreenshot.id);
        if (cached !== undefined) {
            currentFileSize = cached;
        } else {
            currentFileSize = null;
            const requestId = ++fileSizeRequestId;
            const screenshotId = currentScreenshot.id;
            const filename = currentScreenshot.filename;
            stat(filename)
                .then((s) => {
                    fileSizeCache.set(screenshotId, s.size);
                    if (
                        requestId === fileSizeRequestId
                        && currentScreenshot?.id === screenshotId
                    ) {
                        currentFileSize = s.size;
                    }
                })
                .catch(() => {
                    fileSizeCache.set(screenshotId, null);
                    if (
                        requestId === fileSizeRequestId
                        && currentScreenshot?.id === screenshotId
                    ) {
                        currentFileSize = null;
                    }
                });
        }
    } else {
        currentFileSize = null;
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
        if (isFullscreenViewer) {
            void setFullscreenViewer(false);
        }
        viewMode = "grid";
        isZoomed = false;
    };

    const handleViewerImageReady = () => {
        viewerImageReady = true;
    };

    const scheduleFullscreenUiHide = () => {
        if (fullscreenUiTimer) {
            clearTimeout(fullscreenUiTimer);
            fullscreenUiTimer = null;
        }
        if (!isFullscreenViewer) return;
        fullscreenUiTimer = setTimeout(() => {
            showFullscreenCursor = false;
            showFullscreenChrome = false;
            showFullscreenFilmstrip = false;
            fullscreenUiTimer = null;
        }, 1600);
    };

    const hideFullscreenUi = () => {
        if (fullscreenUiTimer) {
            clearTimeout(fullscreenUiTimer);
            fullscreenUiTimer = null;
        }
        showFullscreenCursor = false;
        showFullscreenChrome = false;
        showFullscreenFilmstrip = false;
    };

    const revealFullscreenUi = () => {
        showFullscreenCursor = true;
        scheduleFullscreenUiHide();
    };

    const revealFullscreenChrome = () => {
        showFullscreenCursor = true;
        showFullscreenChrome = true;
        showFullscreenFilmstrip = false;
        scheduleFullscreenUiHide();
    };

    const revealFullscreenFilmstrip = () => {
        showFullscreenCursor = true;
        showFullscreenFilmstrip = true;
        showFullscreenChrome = false;
        scheduleFullscreenUiHide();
        fullscreenFilmstripApi?.reInit();
    };

    const setFullscreenViewer = async (nextState: boolean) => {
        if (isFullscreenViewer === nextState) return;
        try {
            await currentWindowHandle?.setFullscreen(nextState);
            isFullscreenViewer = nextState;
            showFullscreenCursor = nextState;
            showFullscreenChrome = false;
            showFullscreenFilmstrip = false;
            if (nextState) {
                scheduleFullscreenUiHide();
            } else if (fullscreenUiTimer) {
                clearTimeout(fullscreenUiTimer);
                fullscreenUiTimer = null;
            }
        } catch (e) {
            console.error("Failed to toggle screenshot fullscreen", e);
        }
    };

    const toggleFullscreenViewer = () => {
        void setFullscreenViewer(!isFullscreenViewer);
    };

    const handleViewerBackdropClick = () => {
        if (isFullscreenViewer) {
            scheduleFullscreenUiHide();
            return;
        }
        backToGrid();
    };

    const handleGameSelect = (id: number | null) => {
        selectedGameId = id;
        viewMode = "grid";
        if (gridScrollContainer) {
            gridScrollContainer.scrollTop = 0;
        }
        gridScrollTop = 0;
    };

    $: gamesById = new Map(allGames.map((g) => [g.id, g]));

    $: currentGame = currentScreenshot
        ? (gamesById.get(currentScreenshot.gameId) ?? null)
        : null;

    $: currentGameName = currentGame?.gamename || null;

    $: currentGameIcon = currentGame?.icon
        ? convertFileSrc(currentGame.icon)
        : null;

    $: currentScreenshotSrc = currentScreenshot
        ? convertFileSrc(currentScreenshot.filename)
        : null;

    $: currentScreenshotCreatedAtText = currentScreenshot
        ? new Date(currentScreenshot.createdAt).toLocaleString("ja-JP")
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

    const selectViewerIndex = (index: number) => {
        currentIndex = index;
        if (isFullscreenViewer) revealFullscreenFilmstrip();
    };

    const horizontalWheelScroll = (node: HTMLElement) => {
        const handleWheel = (event: WheelEvent) => {
            event.preventDefault();
            event.stopPropagation();
            const delta = event.deltaX || event.deltaY;
            if (delta > 0) {
                fullscreenFilmstripApi?.scrollNext();
            } else if (delta < 0) {
                fullscreenFilmstripApi?.scrollPrev();
            }
            revealFullscreenFilmstrip();
        };

        node.addEventListener("wheel", handleWheel, { passive: false });

        return {
            destroy() {
                node.removeEventListener("wheel", handleWheel);
            },
        };
    };

    const handleFullscreenFilmstripInit = (
        event: CustomEvent<EmblaCarouselType>,
    ) => {
        fullscreenFilmstripApi = event.detail;
        fullscreenFilmstripApi.scrollTo(currentIndex, true);
    };

    const handleKeydown = (e: KeyboardEvent) => {
        if (viewMode !== "viewer") return;
        if (e.key === "Escape") {
            if (isFullscreenViewer) {
                void setFullscreenViewer(false);
            } else {
                backToGrid();
            }
        }
        if (e.key === "ArrowRight") next();
        if (e.key === "ArrowLeft") prev();
        if (e.key.toLowerCase() === "f") toggleFullscreenViewer();
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
                const idsToDelete = [...selectedIds];
                await Promise.all(
                    idsToDelete.map((id) => commandDeleteScreenshot(id)),
                );
                const deletedIdsSet = new Set(idsToDelete);
                allScreenshots = allScreenshots.filter(
                    (s) => !deletedIdsSet.has(s.id),
                );
                for (const id of deletedIdsSet) {
                    fileSizeCache.delete(id);
                }
                selectedIds.clear();
                selectedIds = selectedIds;
                selectionMode = false;
            } catch (e) {
                console.error("Failed to batch delete screenshots", e);
            }
        } else if (currentScreenshot && viewMode === "viewer") {
            try {
                await commandDeleteScreenshot(currentScreenshot.id);
                fileSizeCache.delete(currentScreenshot.id);
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

    $: fullscreenFilmstripScreenshots = viewerScreenshots.map((screenshot) => ({
        screenshot,
        src: convertFileSrc(screenshot.thumbnailFilename ?? screenshot.filename),
    }));

    $: if (fullscreenFilmstripApi && showFullscreenFilmstrip) {
        fullscreenFilmstripApi.reInit();
        fullscreenFilmstripApi.scrollTo(currentIndex, true);
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
    class="h-screen w-screen bg-bg-primary text-text-primary flex flex-col overflow-hidden"
>
    {#if isBootstrapping}
        <TitleBar variant="screenshot" heightClass="h-12" />
        <div class="flex-1 min-h-0 bg-black" />
    {:else if viewMode === "viewer" && currentScreenshot}
        {#if !isFullscreenViewer}
            <TitleBar variant="screenshot" heightClass="h-12">
                <div slot="left" class="flex items-center pl-1 pr-4 shrink-0">
                    <button
                        class="flex items-center gap-0 px-1 py-1.5 cursor-pointer outline-none bg-transparent border-none text-white/60 hover:text-white/80 transition-colors text-[16px] font-medium h-fit"
                        on:click={backToGrid}
                    >
                        <span class="i-material-symbols-chevron-left text-2xl" />
                        <span>戻る</span>
                    </button>
                </div>
                <div slot="center" class="flex items-center space-x-2 pointer-events-none">
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
            </TitleBar>
        {/if}


        <!-- Image Content -->
        <div
            class="flex-1 min-h-0 relative bg-black flex items-center justify-center"
            class:fullscreen-viewer-stage={isFullscreenViewer}
            class:fullscreen-cursor-hidden={isFullscreenViewer && !showFullscreenCursor && !showFullscreenChrome && !showFullscreenFilmstrip}
            on:click|self={handleViewerBackdropClick}
            on:mousemove={isFullscreenViewer ? revealFullscreenUi : undefined}
            role="button"
            tabindex="0"
            on:keydown={(e) => {
                if (e.key !== "Escape") return;
                if (isFullscreenViewer) {
                    void setFullscreenViewer(false);
                } else {
                    backToGrid();
                }
            }}
        >
            {#if isFullscreenViewer}
                <div
                    class="absolute inset-x-0 top-0 z-60 h-28"
                    on:mouseenter={revealFullscreenChrome}
                    on:mouseleave={hideFullscreenUi}
                >
                    <button
                        type="button"
                        class="absolute left-1/2 top-0 inline-flex h-12 w-12 -translate-x-1/2 items-center justify-center rounded-full bg-black/70 text-white/90 shadow-2xl ring-1 ring-white/15 backdrop-blur transition-all duration-200 ease-out hover:bg-black/85 hover:text-white"
                        class:translate-y-7={showFullscreenChrome}
                        class:-translate-y-16={!showFullscreenChrome}
                        aria-label="全画面を解除"
                        on:click|stopPropagation={() => void setFullscreenViewer(false)}
                    >
                        <div class="i-material-symbols-close-rounded h-6 w-6" />
                    </button>
                </div>
            {/if}

            <!-- Navigation arrows -->
            <button
                class="absolute left-0 top-0 h-full px-6 flex items-center justify-center text-white bg-black/0 hover:bg-black/30 transition-all z-40 opacity-0 hover:opacity-100"
                class:!hidden={isZoomed || viewerScreenshots.length <= 1}
                on:click|stopPropagation={prev}
            >
                <div
                    class="i-material-symbols-chevron-left text-6xl drop-shadow-lg"
                />
            </button>

            <button
                class="absolute right-0 top-0 h-full px-6 flex items-center justify-center text-white bg-black/0 hover:bg-black/30 transition-all z-40 opacity-0 hover:opacity-100"
                class:!hidden={isZoomed || viewerScreenshots.length <= 1}
                on:click|stopPropagation={next}
            >
                <div
                    class="i-material-symbols-chevron-right text-6xl drop-shadow-lg"
                />
            </button>

            <ZoomableImage
                bind:isZoomed
                src={currentScreenshotSrc ?? convertFileSrc(currentScreenshot.filename)}
                alt={currentScreenshot.filename}
                class="max-w-full max-h-full"
                on:ready={handleViewerImageReady}
            />

            {#if isFullscreenViewer && viewerScreenshots.length > 1}
                <div
                    class="absolute inset-x-0 bottom-0 z-50 px-4 pb-4 pt-10"
                    on:mouseenter={revealFullscreenFilmstrip}
                    on:mouseleave={hideFullscreenUi}
                >
                    <div
                        class="mx-auto max-w-[min(92vw,1200px)] overflow-hidden rounded-xl bg-black/55 p-2 shadow-2xl ring-1 ring-white/10 backdrop-blur transition-all duration-200 cursor-grab active:cursor-grabbing"
                        class:translate-y-0={showFullscreenFilmstrip}
                        class:opacity-100={showFullscreenFilmstrip}
                        class:translate-y-28={!showFullscreenFilmstrip}
                        class:opacity-0={!showFullscreenFilmstrip}
                        use:horizontalWheelScroll
                        use:emblaCarouselSvelte={{ options: fullscreenFilmstripOptions, plugins: [] }}
                        on:emblaInit={handleFullscreenFilmstripInit}
                    >
                        <div class="flex gap-2">
                            {#each fullscreenFilmstripScreenshots as item, index (item.screenshot.id)}
                                <button
                                    type="button"
                                    class="h-16 w-28 shrink-0 overflow-hidden rounded border bg-bg-secondary transition-all {index === currentIndex ? 'border-white opacity-100' : 'border-white/20 opacity-60 hover:opacity-100'}"
                                    aria-label={`${index + 1}枚目を表示`}
                                    on:click|stopPropagation={() => selectViewerIndex(index)}
                                >
                                    <img
                                        src={item.src}
                                        alt=""
                                        class="h-full w-full object-cover"
                                        loading="lazy"
                                        decoding="async"
                                    />
                                </button>
                            {/each}
                        </div>
                    </div>
                </div>
            {/if}
        </div>

        <!-- Bottom Bar -->
        {#if !isFullscreenViewer}
            <div
                class="h-12 flex items-center px-4 shrink-0 bg-[#1a1a1c] border-t border-[#333] justify-between z-10"
                class:opacity-0={isZoomed}
                class:pointer-events-none={isZoomed}
            >
                <div class="text-sm text-[#8b8b8b]">
                    撮影: {currentScreenshotCreatedAtText}
                    {#if currentFileSize !== null}
                        <span class="mx-2">•</span>
                        {formatFileSize(currentFileSize)}
                    {/if}
                </div>

                <div class="flex items-center space-x-2">
                    <button
                        class="flex items-center space-x-1 px-3 py-1.5 rounded bg-[#333] text-white hover:bg-[#444] transition-colors"
                        on:click={toggleFullscreenViewer}
                        title="全画面"
                    >
                        <span class="i-material-symbols-fullscreen-rounded text-lg" />
                    </button>
                    <button
                        class="flex items-center space-x-1 px-3 py-1.5 rounded bg-[#333] text-white hover:bg-[#444] transition-colors"
                        on:click={confirmDelete}
                        title="削除"
                    >
                        <span class="i-material-symbols-delete text-lg" />
                    </button>
                </div>
            </div>
        {/if}
    {:else}
        <!-- Grid Mode -->
        <TitleBar variant="screenshot" heightClass="h-14">
            <div slot="left" class="flex items-center space-x-3 px-4 shrink-0">
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

            <div slot="right" class="flex items-center gap-2 pr-2 shrink-0">
                {#if selectionMode}
                    <button
                        class="px-3 py-1.5 text-sm rounded bg-bg-tertiary text-text-primary hover:bg-bg-button transition-colors h-fit"
                        on:click={toggleSelectionMode}
                    >
                        キャンセル
                    </button>
                    <button
                        class="px-3 py-1.5 text-sm rounded transition-colors h-fit {selectedIds.size >
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
                        class="px-3 py-1.5 text-sm rounded bg-bg-tertiary text-text-primary hover:bg-bg-button transition-colors flex items-center gap-1 h-fit"
                        on:click={toggleSelectionMode}
                        title="一括編集"
                    >
                        <span class="i-material-symbols-checklist text-lg" />
                        選択
                    </button>
                {/if}
            </div>
        </TitleBar>

        <!-- Content -->
        <div class="flex-1 min-h-0 relative">
            <div
                class="h-full w-full overflow-y-auto p-4 custom-scrollbar"
                use:observeGridContainer
            >
                <ScreenshotGrid
                    screenshots={filteredScreenshots}
                    {selectionMode}
                    {selectedIds}
                    scrollTop={gridScrollTop}
                    viewportHeight={gridViewportHeight}
                    contentWidth={gridContentWidth}
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

<style>
    .fullscreen-viewer-stage.fullscreen-cursor-hidden,
    .fullscreen-viewer-stage.fullscreen-cursor-hidden :global(*) {
        cursor: none !important;
    }
</style>
