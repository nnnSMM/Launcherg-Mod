<script lang="ts">
    import { onMount, createEventDispatcher } from "svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
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

            for (const path of paths) {
                await commandImportScreenshot(gameId, path);
            }

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

<div class="w-full bg-bg-secondary/30 p-2 lg:p-4">
    <div class="flex flex-col gap-3 mb-4">
        <h2 class="text-xl font-bold text-text-primary">
            スクリーンショット <span
                class="text-sm font-normal text-text-secondary ml-2"
                >({screenshots.length})</span
            >
        </h2>
        <div class="flex items-center gap-2">
            <button
                on:click={handleImport}
                class="flex items-center gap-1 px-3 py-1.5 bg-bg-tertiary text-text-primary hover:bg-bg-button transition-colors text-sm rounded"
                title="インポート"
            >
                <div class="i-material-symbols-upload text-lg" />
            </button>
            <button
                on:click={() => openViewer()}
                class="flex items-center gap-1 px-3 py-1.5 bg-accent-accent text-white hover:bg-accent-accent/80 transition-colors text-sm whitespace-nowrap rounded"
            >
                <div class="i-material-symbols-open-in-new text-lg" />
                すべて見る
            </button>
        </div>
    </div>

    {#if screenshots.length === 0}
        <div
            class="text-center text-text-secondary py-12 text-sm bg-bg-primary/50"
        >
            まだスクリーンショットを撮影していません。<br
            />Windows+Shift+SやWindows+PrintScreenを押して撮影してください。
        </div>
    {:else}
        <div class="grid grid-cols-2 gap-2">
            {#each previewScreenshots as screenshot}
                <div class="relative group/tooltip">
                    <button
                        class="w-full relative aspect-video group overflow-hidden transition-all cursor-pointer shadow-sm hover:shadow-md"
                        on:click={() => openViewer(screenshot)}
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
