<script lang="ts">
    import { onMount } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import type { Screenshot, CollectionElement } from "@/lib/types";
    import {
        commandGetAllScreenshots,
        commandGetAllElements,
    } from "@/lib/command";
    import ScreenshotGrid from "@/components/ScreenshotWindow/ScreenshotGrid.svelte";
    import ScreenshotViewer from "@/components/Work/ScreenshotViewer.svelte";

    // Custom simple dropdown for now if generic one is not suitable
    import GameSelector from "@/components/ScreenshotWindow/GameSelector.svelte";

    let allScreenshots: Screenshot[] = [];
    let allGames: CollectionElement[] = [];

    let viewMode: "grid" | "viewer" = "grid";
    let selectedGameId: number | null = null;

    // For viewer
    let viewerScreenshots: Screenshot[] = [];
    let viewerInitialIndex: number = 0;

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
        console.log(
            "handleArgs called with:",
            args,
            "screenshots count:",
            screenshots.length,
        );

        // Always set the game filter if provided
        if (args.game_id !== undefined && args.game_id !== null) {
            selectedGameId = args.game_id;
        }

        // If initial_screenshot_id is provided, switch to viewer mode
        if (args.initial_screenshot_id) {
            const target = screenshots.find(
                (s) => s.id === args.initial_screenshot_id,
            );
            console.log(
                "Looking for screenshot id:",
                args.initial_screenshot_id,
                "found:",
                target,
            );

            if (target) {
                // Need to recalculate filtered screenshots based on the new selectedGameId
                const currentFiltered = selectedGameId
                    ? screenshots.filter((s) => s.gameId === selectedGameId)
                    : screenshots;

                viewerScreenshots = currentFiltered;
                const idx = viewerScreenshots.findIndex(
                    (s) => s.id === args.initial_screenshot_id,
                );
                viewerInitialIndex = idx >= 0 ? idx : 0;
                viewMode = "viewer";
                console.log(
                    "Switched to viewer mode, index:",
                    viewerInitialIndex,
                );
            }
        }
    };

    onMount(async () => {
        console.log("[ScreenshotWindow] onMount started");
        const freshScreenshots = await fetchData();
        console.log(
            "[ScreenshotWindow] Fetched",
            freshScreenshots.length,
            "screenshots",
        );

        // Check for initial args set by backend script injection
        const w = window as any;
        if (w.__INITIAL_SCREENSHOT_ARGS__) {
            console.log(
                "[ScreenshotWindow] Found initial args:",
                w.__INITIAL_SCREENSHOT_ARGS__,
            );
            handleArgs(w.__INITIAL_SCREENSHOT_ARGS__, freshScreenshots);
            w.__INITIAL_SCREENSHOT_ARGS__ = null; // consume
        }

        // Use getCurrentWindow().listen for window-specific events
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const currentWindow = getCurrentWindow();
        console.log(
            "[ScreenshotWindow] Setting up listener on window:",
            currentWindow.label,
        );

        const unlisten = await currentWindow.listen<WindowArgs>(
            "screenshot-window-args",
            async (event) => {
                console.log(
                    "[ScreenshotWindow] Received screenshot-window-args event:",
                    event.payload,
                );
                // Refresh data first in case new screenshots were added
                const freshData = await fetchData();
                // Then handle the args with fresh data
                handleArgs(event.payload, freshData);
            },
        );

        // Handle window close to properly destroy the window
        const unlistenClose = await currentWindow.onCloseRequested(
            async (event) => {
                console.log(
                    "[ScreenshotWindow] Close requested, destroying window",
                );
                // Prevent default close behavior (which might just hide)
                event.preventDefault();
                // Destroy the window
                await currentWindow.destroy();
            },
        );

        console.log("[ScreenshotWindow] Listener registered successfully");

        return () => {
            console.log("[ScreenshotWindow] Cleaning up listener");
            unlisten();
            unlistenClose();
        };
    });

    $: filteredScreenshots = selectedGameId
        ? allScreenshots.filter((s) => s.gameId === selectedGameId)
        : allScreenshots;

    const enterViewer = (targetScreenshot: Screenshot) => {
        viewerScreenshots = filteredScreenshots;
        const idx = viewerScreenshots.findIndex(
            (s) => s.id === targetScreenshot.id,
        );
        viewerInitialIndex = idx >= 0 ? idx : 0;
        viewMode = "viewer";
    };

    const backToGrid = () => {
        viewMode = "grid";
    };

    const handleGameSelect = (id: number | null) => {
        selectedGameId = id;
        viewMode = "grid"; // Reset to grid when changing game
    };

    $: currentGameName = selectedGameId
        ? allGames.find((g) => g.id === selectedGameId)?.gamename
        : "All Media";
</script>

<div
    class="h-screen w-screen bg-bg-primary text-text-primary flex flex-col overflow-hidden"
>
    <!-- Header -->
    <div
        class="h-14 border-b border-border-primary flex items-center px-4 shrink-0 bg-bg-secondary justify-between z-10"
    >
        <div class="flex items-center space-x-4">
            {#if viewMode === "viewer"}
                <button
                    class="flex items-center space-x-1 text-text-secondary hover:text-text-primary transition-colors hover:bg-bg-tertiary px-3 py-1.5 rounded"
                    on:click={backToGrid}
                >
                    <span class="i-material-symbols-arrow-back text-xl" />
                    <span class="font-bold">Back</span>
                </button>
                <div class="flex items-center space-x-2">
                    <!-- Maybe show game icon? -->
                    <span class="font-bold text-sm truncate max-w-md"
                        >{currentGameName}</span
                    >
                </div>
            {:else}
                <div class="flex items-center">
                    <GameSelector
                        games={gamesWithScreenshots}
                        {selectedGameId}
                        on:select={(e) => handleGameSelect(e.detail)}
                    />
                </div>
            {/if}
        </div>
    </div>

    <!-- Content -->
    <div class="flex-1 min-h-0 relative">
        {#if viewMode === "viewer"}
            <ScreenshotViewer
                bind:screenshots={viewerScreenshots}
                initialIndex={viewerInitialIndex}
                on:close={backToGrid}
            />
        {:else}
            <!-- Update ScreenshotGrid to dispatch click event instead of internal handling if we want centralized logic -->
            <div class="h-full w-full overflow-y-auto p-4 custom-scrollbar">
                <ScreenshotGrid
                    screenshots={filteredScreenshots}
                    on:reload={fetchData}
                    on:clickScreenshot={(e) => enterViewer(e.detail)}
                />
            </div>
        {/if}
    </div>
</div>
