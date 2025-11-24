<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { Screenshot } from "@/lib/types";
    import { commandDeleteScreenshot } from "@/lib/command";

    export let screenshots: Screenshot[];
    export let initialIndex: number;

    let currentIndex = initialIndex;
    let currentScreenshot: Screenshot;
    let showDeleteConfirm = false;

    const dispatch = createEventDispatcher();

    $: {
        if (screenshots.length > 0) {
            if (currentIndex >= screenshots.length)
                currentIndex = screenshots.length - 1;
            if (currentIndex < 0) currentIndex = 0;

            currentScreenshot = screenshots[currentIndex];
        } else {
            dispatch("close");
        }
    }

    const next = () => {
        currentIndex = (currentIndex + 1) % screenshots.length;
    };

    const prev = () => {
        currentIndex =
            (currentIndex - 1 + screenshots.length) % screenshots.length;
    };

    const close = () => {
        dispatch("close");
    };

    const confirmDelete = () => {
        showDeleteConfirm = true;
    };

    const cancelDelete = () => {
        showDeleteConfirm = false;
    };

    const handleDelete = async () => {
        showDeleteConfirm = false;
        try {
            await commandDeleteScreenshot(currentScreenshot.id);
            screenshots = screenshots.filter(
                (s) => s.id !== currentScreenshot.id,
            );
            if (screenshots.length === 0) {
                dispatch("close");
            } else if (currentIndex >= screenshots.length) {
                currentIndex = screenshots.length - 1;
            }
        } catch (e) {
            console.error("Failed to delete screenshot", e);
        }
    };

    const handleKeydown = (e: KeyboardEvent) => {
        if (e.key === "Escape") close();
        if (e.key === "ArrowRight") next();
        if (e.key === "ArrowLeft") prev();
    };
</script>

<svelte:window on:keydown={handleKeydown} />

<div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-sm"
    on:click|self={close}
    on:keydown={(e) => e.key === "Escape" && close()}
    role="button"
    tabindex="0"
>
    <button
        class="absolute top-4 right-4 p-3 text-text-primary bg-bg-tertiary/80 hover:bg-bg-tertiary rounded-full transition-all backdrop-blur-sm z-50 shadow-lg"
        on:click={close}
    >
        <div class="i-material-symbols-close text-3xl" />
    </button>

    <button
        class="absolute left-0 top-0 h-full px-6 flex items-center justify-center text-white bg-black/0 hover:bg-black/30 transition-all opacity-0 hover:opacity-100 z-40"
        on:click={prev}
    >
        <div class="i-material-symbols-chevron-left text-6xl drop-shadow-lg" />
    </button>

    <button
        class="absolute right-0 top-0 h-full px-6 flex items-center justify-center text-white bg-black/0 hover:bg-black/30 transition-all opacity-0 hover:opacity-100 z-40"
        on:click={next}
    >
        <div class="i-material-symbols-chevron-right text-6xl drop-shadow-lg" />
    </button>

    <div class="flex flex-col items-center max-w-[90vw] max-h-[90vh]">
        <img
            src={convertFileSrc(currentScreenshot.filename)}
            alt={currentScreenshot.filename}
            class="max-w-full max-h-[80vh] object-contain shadow-2xl rounded-lg"
        />

        <div
            class="mt-4 w-full flex items-center justify-between text-white relative z-50"
        >
            <div class="text-sm opacity-70">
                {new Date(currentScreenshot.createdAt).toLocaleString()}
            </div>

            <button
                class="flex items-center gap-2 px-3 py-1.5 rounded bg-red-500/20 hover:bg-red-500/40 text-red-200 hover:text-red-100 transition-colors text-sm"
                on:click={confirmDelete}
            >
                <div class="i-material-symbols-delete text-xl" />
                削除
            </button>
        </div>
    </div>

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
                on:click|stopPropagation
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
