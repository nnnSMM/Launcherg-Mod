<script lang="ts">
    import { onMount } from "svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { listen } from "@tauri-apps/api/event";
    import type { Screenshot } from "@/lib/types";
    import {
        commandGetGameScreenshots,
        commandImportScreenshot,
        commandOpenScreenshotWindow,
    } from "@/lib/command";
    import { getFriendlyErrorMessage, reportError } from "@/lib/errors";
    import { showErrorToast } from "@/lib/toast";

    export let gameId: number;

    const isDemoBuild = import.meta.env.BASE_URL === "./";

    let screenshots: Screenshot[] = [];
    let gridWidth = 0;
    let viewportHeight = 0;
    let currentPage = 1;
    const TILE_GAP = 6;
    const APP_CHROME_HEIGHT = 80;
    const GALLERY_HEADER_HEIGHT = 58;
    const GALLERY_FOOTER_HEIGHT = 58;
    const GRID_VERTICAL_PADDING = 32;
    const BOTTOM_BREATHING_ROOM = 44;
    const MIN_PREVIEW_ROWS = 2;
    const MAX_PREVIEW_COLUMNS = 8;
    const MIN_TILE_WIDTH = 240;
    const MAX_TILE_WIDTH = 340;

    const getTileHeight = (width: number, columns: number) => {
        if (!width || columns <= 0) return 1;
        const tileWidth = (width - TILE_GAP * (columns - 1)) / columns;
        return Math.max(1, tileWidth * 9 / 16);
    };

    const getFitRows = (height: number, tileHeight: number) =>
        Math.max(1, Math.floor((height + TILE_GAP) / (tileHeight + TILE_GAP)));

    const choosePreviewColumns = (width: number, height: number) => {
        if (!width || !height) return 2;

        const minColumns = Math.max(
            2,
            Math.ceil((width + TILE_GAP) / (MAX_TILE_WIDTH + TILE_GAP)),
        );
        const maxColumns = Math.max(
            minColumns,
            Math.min(
                MAX_PREVIEW_COLUMNS,
                Math.floor((width + TILE_GAP) / (MIN_TILE_WIDTH + TILE_GAP)),
            ),
        );

        for (let columns = minColumns; columns <= maxColumns; columns++) {
            const rows = getFitRows(height, getTileHeight(width, columns));
            if (rows < MIN_PREVIEW_ROWS) {
                continue;
            }
            return columns;
        }

        return maxColumns;
    };

    onMount(async () => {
        await loadScreenshots();

        const unlisten = listen<number>("collection-element-updated", (event) => {
            if (event.payload === gameId) {
                loadScreenshots();
            }
        });

        return () => {
            unlisten.then((fn) => fn());
        };
    });

    const loadScreenshots = async () => {
        try {
            screenshots = await commandGetGameScreenshots(gameId);
        } catch (e) {
            reportError("screenshots.load", e);
        }
    };

    const handleImport = async () => {
        if (isDemoBuild) return;
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

            await Promise.all(
                paths.map((path) => commandImportScreenshot(gameId, path)),
            );

            await loadScreenshots();
        } catch (e) {
            reportError("screenshots.import", e);
            showErrorToast(getFriendlyErrorMessage(e, "スクリーンショットのインポートに失敗しました"));
        }
    };

    const openViewer = async (screenshot?: Screenshot) => {
        await commandOpenScreenshotWindow(
            gameId,
            screenshot?.id,
            screenshot,
        );
    };

    const formatCreatedAt = (value: string) =>
        new Date(value).toLocaleDateString("ja-JP", {
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
        });

    $: sortedScreenshots = [...screenshots].sort(
        (a, b) =>
            new Date(b.createdAt).getTime() -
            new Date(a.createdAt).getTime(),
    );
    $: availableGalleryHeight = Math.max(
        1,
        viewportHeight -
            APP_CHROME_HEIGHT -
            GALLERY_HEADER_HEIGHT -
            GRID_VERTICAL_PADDING -
            BOTTOM_BREATHING_ROOM,
    );
    $: previewColumns = choosePreviewColumns(gridWidth, availableGalleryHeight);
    $: tileHeight = getTileHeight(gridWidth, previewColumns);
    $: rowsWithoutFooter = Math.max(
        1,
        Math.floor((availableGalleryHeight + TILE_GAP) / (tileHeight + TILE_GAP)),
    );
    $: needsFooter = screenshots.length > previewColumns * rowsWithoutFooter;
    $: availableRowsHeight = needsFooter
        ? Math.max(tileHeight, availableGalleryHeight - GALLERY_FOOTER_HEIGHT)
        : availableGalleryHeight;
    $: previewRows = Math.max(
        1,
        Math.floor((availableRowsHeight + TILE_GAP) / (tileHeight + TILE_GAP)),
    );
    $: pageSize = previewColumns * previewRows;
    $: pageCount = Math.max(1, Math.ceil(screenshots.length / pageSize));
    $: if (currentPage > pageCount) currentPage = pageCount;
    $: if (currentPage < 1) currentPage = 1;
    $: pageStart = (currentPage - 1) * pageSize;
    $: pageEnd = Math.min(pageStart + pageSize, screenshots.length);
    $: previewScreenshots = sortedScreenshots.slice(pageStart, pageEnd);
    $: hasMultiplePages = pageCount > 1;
    $: visibleRangeText =
        screenshots.length === 0
            ? "0"
            : `${pageStart + 1}-${pageEnd} / ${screenshots.length}`;

    const goToPrevPage = () => {
        currentPage = Math.max(1, currentPage - 1);
    };

    const goToNextPage = () => {
        currentPage = Math.min(pageCount, currentPage + 1);
    };
</script>

<svelte:window bind:innerHeight={viewportHeight} />

<div class="w-full rounded-lg border border-border-primary bg-bg-primary/56 shadow-sm backdrop-blur-md">
    <div
        class="flex flex-col gap-3 border-b border-border-primary/70 px-4 py-3 sm:flex-row sm:items-center sm:justify-between lg:px-5"
    >
        <div class="flex min-w-0 items-center gap-2">
            <div class="i-material-symbols-image-outline h-5 w-5 shrink-0 color-ui-tertiary" />
            <h2 class="truncate text-h3 font-bold text-text-primary">
                最近のスクリーンショット
            </h2>
            <span
                class="rounded border border-border-primary bg-bg-secondary/30 px-2 py-0.5 text-caption font-semibold text-text-secondary"
            >
                {visibleRangeText}
            </span>
        </div>
        <div class="flex items-center gap-2">
            {#if hasMultiplePages}
                <div class="mr-1 flex items-center gap-1">
                    <button
                        type="button"
                        on:click={goToPrevPage}
                        disabled={currentPage === 1}
                        aria-label="前のページ"
                        class="inline-flex h-8 w-8 items-center justify-center rounded border border-border-primary bg-bg-button/20 text-text-primary transition-colors hover:bg-bg-button-hover disabled:cursor-not-allowed disabled:opacity-40"
                    >
                        <div class="i-material-symbols-chevron-left-rounded h-5 w-5" />
                    </button>
                    <div class="min-w-12 text-center text-caption font-semibold text-text-secondary">
                        {currentPage} / {pageCount}
                    </div>
                    <button
                        type="button"
                        on:click={goToNextPage}
                        disabled={currentPage === pageCount}
                        aria-label="次のページ"
                        class="inline-flex h-8 w-8 items-center justify-center rounded border border-border-primary bg-bg-button/20 text-text-primary transition-colors hover:bg-bg-button-hover disabled:cursor-not-allowed disabled:opacity-40"
                    >
                        <div class="i-material-symbols-chevron-right-rounded h-5 w-5" />
                    </button>
                </div>
            {/if}
            <button
                type="button"
                on:click={handleImport}
                aria-label="スクリーンショットをインポート"
                class="inline-flex h-8 items-center gap-1.5 rounded border border-border-primary bg-bg-button/25 px-2.5 text-body3 font-medium text-text-primary transition-colors hover:bg-bg-button-hover focus-visible:ring-2 focus-visible:ring-accent-accent"
                title="インポート"
            >
                <div class="i-material-symbols-upload h-4 w-4" />
                インポート
            </button>
            <button
                type="button"
                on:click={() => openViewer()}
                aria-label="スクリーンショット一覧を開く"
                class="inline-flex h-8 items-center gap-1.5 whitespace-nowrap rounded bg-accent-accent px-2.5 text-body3 font-medium text-white transition-colors hover:bg-accent-accent/80 focus-visible:ring-2 focus-visible:ring-accent-accent"
            >
                <div class="i-material-symbols-open-in-new h-4 w-4" />
                一覧
            </button>
        </div>
    </div>

    {#if screenshots.length === 0}
        <div
            class="m-4 rounded-lg border border-dashed border-border-primary bg-bg-secondary/20 px-4 py-12 text-center text-body3 text-text-secondary lg:m-5"
        >
            まだスクリーンショットはありません。
        </div>
    {:else}
        <div
            bind:clientWidth={gridWidth}
            class="grid gap-1.5 p-3 lg:p-4"
            style="grid-template-columns: repeat({previewColumns}, minmax(0, 1fr));"
        >
            {#each previewScreenshots as screenshot (screenshot.id)}
                <div class="min-w-0">
                    <button
                        type="button"
                        class="group relative aspect-video w-full cursor-pointer overflow-hidden rounded border border-border-primary/60 bg-bg-secondary/20 shadow-sm transition-all hover:-translate-y-0.5 hover:border-accent-accent/70 hover:shadow-lg focus-visible:ring-2 focus-visible:ring-accent-accent"
                        on:click={() => openViewer(screenshot)}
                        aria-label={`${screenshot.filename}を開く`}
                    >
                        <img
                            src={convertFileSrc(
                                screenshot.thumbnailFilename ??
                                    screenshot.filename,
                            )}
                            alt={screenshot.filename}
                            class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
                            loading="lazy"
                        />
                        <div
                            class="absolute inset-0 bg-gradient-to-t from-black/65 via-black/0 to-black/0 opacity-75 transition-opacity group-hover:opacity-100"
                        />
                        <div
                            class="absolute inset-x-0 bottom-0 flex items-end justify-between gap-2 p-2"
                        >
                            <div
                                class="min-w-0 truncate text-left text-[11px] font-medium leading-tight text-white/90"
                            >
                                {formatCreatedAt(screenshot.createdAt)}
                            </div>
                            <div
                                class="i-material-symbols-open-in-new-rounded h-4 w-4 shrink-0 text-white/90 opacity-0 transition-opacity group-hover:opacity-100"
                            />
                        </div>
                    </button>
                </div>
            {/each}
        </div>
        {#if hasMultiplePages}
            <div
                class="flex flex-col gap-2 border-t border-border-primary/70 px-4 py-3 text-body3 text-text-secondary sm:flex-row sm:items-center sm:justify-between lg:px-5"
            >
                <span>
                    {pageStart + 1}-{pageEnd} 件目を表示中です。
                </span>
            </div>
        {/if}
    {/if}
</div>
