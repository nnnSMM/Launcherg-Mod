<script lang="ts">
    import { onMount, createEventDispatcher } from "svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { listen } from "@tauri-apps/api/event";
    import type { Screenshot } from "@/lib/types";
    import {
        commandGetGameScreenshots,
        commandImportScreenshot,
        commandOpenScreenshotWindow,
    } from "@/lib/command";

    export let gameId: number;

    let screenshots: Screenshot[] = [];

    // 表示する最新スクリーンショットの最大数
    const PREVIEW_LIMIT = 4;

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
            console.error("Failed to load screenshots", e);
        }
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

            await Promise.all(
                paths.map((path) => commandImportScreenshot(gameId, path)),
            );

            await loadScreenshots();
        } catch (e) {
            console.error("Failed to import screenshot", e);
        }
    };

    const openViewer = async (screenshot?: Screenshot) => {
        await commandOpenScreenshotWindow(
            gameId,
            screenshot?.id,
            screenshot,
        );
    };

    $: previewScreenshots = [...screenshots]
        .sort(
            (a, b) =>
                new Date(b.createdAt).getTime() -
                new Date(a.createdAt).getTime(),
        )
        .slice(0, PREVIEW_LIMIT);
</script>

<div class="w-full rounded-lg border border-border-primary bg-bg-primary/72 shadow-lg p-4 lg:p-5">
    <div class="flex flex-col gap-3 mb-4">
        <div class="flex items-center gap-2">
            <div class="i-material-symbols-image-outline w-5 h-5 color-ui-tertiary" />
            <h2 class="text-h3 font-bold text-text-primary">
                スクリーンショット <span
                    class="text-sm font-normal text-text-secondary ml-2"
                    >({screenshots.length})</span
                >
            </h2>
        </div>
        <div class="flex items-center gap-2">
            <button
                on:click={handleImport}
                aria-label="スクリーンショットをインポート"
                class="flex items-center gap-1 px-3 py-1.5 bg-bg-tertiary text-text-primary hover:bg-bg-button transition-colors text-sm rounded focus-visible:ring-2 focus-visible:ring-accent-accent"
                title="インポート"
            >
                <div class="i-material-symbols-upload text-lg" />
            </button>
            <button
                on:click={() => openViewer()}
                aria-label="スクリーンショット一覧を開く"
                class="flex items-center gap-1 px-3 py-1.5 bg-accent-accent text-white hover:bg-accent-accent/80 transition-colors text-sm whitespace-nowrap rounded focus-visible:ring-2 focus-visible:ring-accent-accent"
            >
                <div class="i-material-symbols-open-in-new text-lg" />
                すべて見る
            </button>
        </div>
    </div>

    {#if screenshots.length === 0}
        <div
            class="text-center text-text-secondary py-12 text-sm bg-bg-secondary/35 rounded-lg border border-dashed border-border-primary"
        >
            まだスクリーンショットを撮影していません。<br
            />Windows+Shift+SやWindows+PrintScreenを押して撮影してください。
        </div>
    {:else}
        <div class="grid grid-cols-2 gap-2">
            {#each previewScreenshots as screenshot (screenshot.id)}
                <div class="relative group/tooltip">
                    <button
                        class="w-full relative aspect-video group overflow-hidden transition-all cursor-pointer shadow-sm hover:shadow-md rounded focus-visible:ring-2 focus-visible:ring-accent-accent"
                        on:click={() => openViewer(screenshot)}
                        aria-label={`${screenshot.filename}を開く`}
                    >
                        <img
                            src={convertFileSrc(
                                screenshot.thumbnailFilename ??
                                    screenshot.filename,
                            )}
                            alt={screenshot.filename}
                            class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                            loading="lazy"
                        />
                        <div
                            class="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors"
                        />
                    </button>

                    <!-- ホバー時ツールチップ -->
                    <div
                        class="pointer-events-none absolute right-[105%] top-1/2 -translate-y-1/2 z-50 w-96 opacity-0 invisible group-hover/tooltip:opacity-100 group-hover/tooltip:visible transition-all duration-75 group-hover/tooltip:duration-200 delay-0 group-hover/tooltip:delay-200"
                    >
                        <!-- 吹き出しの枠 -->
                        <div
                            class="bg-bg-primary border border-border-primary p-2 shadow-2xl"
                        >
                            <img
                                src={convertFileSrc(screenshot.filename)}
                                alt={screenshot.filename}
                                class="w-full aspect-video object-cover mb-2"
                                loading="lazy"
                            />
                            <div
                                class="text-xs text-text-secondary text-center font-mono"
                            >
                                {new Date(
                                    screenshot.createdAt,
                                ).toLocaleString()}
                            </div>
                        </div>
                        <!-- 吹き出しの三角形 (右向き) -->
                        <div
                            class="absolute top-1/2 -right-2 -translate-y-1/2 w-0 h-0 border-y-8 border-y-transparent border-l-8 border-l-border-primary"
                        ></div>
                        <div
                            class="absolute top-1/2 -right-[7px] -translate-y-1/2 w-0 h-0 border-y-[7px] border-y-transparent border-l-[7px] border-l-bg-primary"
                        ></div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
