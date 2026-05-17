<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { CollectionElement } from "@/lib/types";
    import { createEventDispatcher } from "svelte";
    import { commandPlayGame } from "@/lib/command";

    import { open } from "@tauri-apps/plugin-dialog";
    import { commandUpdateGameImage } from "@/lib/command";
    import ContextMenu from "@/components/UI/ContextMenu.svelte";
    import ZoomableImage from "@/components/UI/ZoomableImage.svelte";

    export let element: CollectionElement;
    export let offset: number = 0;
    export let heroHeight: number = 0;

    const dispatch = createEventDispatcher();

    $: bgImage =
        element.thumbnail && element.thumbnail.trim() !== ""
            ? `${convertFileSrc(element.thumbnail)}?v=${element.updatedAt}`
            : "/images/dummy_thumbnail.svg";

    const handleImageError = (e: Event) => {
        const img = e.target as HTMLImageElement;
        img.src = "/images/dummy_thumbnail.svg";
    };

    let heroWidth = 0;

    let menu = {
        isOpen: false,
        x: 0,
        y: 0,
    };

    const handleContextMenu = (e: MouseEvent) => {
        e.preventDefault();
        menu = {
            isOpen: true,
            x: e.clientX,
            y: e.clientY,
        };
    };

    const menuOptions = [
        {
            label: "サムネイルを変更",
            onSelect: async () => {
                const selected = await open({
                    multiple: false,
                    filters: [
                        { name: "Images", extensions: ["png", "jpg", "jpeg"] },
                    ],
                });
                if (typeof selected?.path === "string") {
                    await commandUpdateGameImage(
                        element.id,
                        "thumbnail",
                        selected.path,
                    );
                    element.updatedAt = new Date().toISOString();
                }
            },
        },
    ];

    const handlePlay = async (
        e: CustomEvent<{ isAdmin: boolean | undefined }>,
    ) => {
        try {
            await commandPlayGame(element.id, !!e.detail.isAdmin);
            dispatch("play");
        } catch (error) {
            console.error("Failed to play game:", error);
        }
    };

    let showFullscreenImage = false;
    let innerWidth = 0;

    $: TARGET_AREA = innerWidth < 1280 ? 90000 : 130000;

    $: imageWidth = (() => {
        const width = element.thumbnailWidth || 16;
        const height = element.thumbnailHeight || 9;
        const ratio = width / height;
        return Math.sqrt(TARGET_AREA * ratio);
    })();
</script>

<svelte:window bind:innerWidth />

<div
    bind:clientWidth={heroWidth}
    bind:clientHeight={heroHeight}
    class="relative w-full min-h-[60vh] min-h-[300px] group flex flex-col pointer-events-none"
>
    <!-- Content -->
    <div
        class="relative z-10 flex-1 flex flex-col justify-between pt-12 px-12 pb-24 pointer-events-none"
    >
        <!-- Top: Title Section -->
        <div
            class="flex items-start justify-between gap-12 pointer-events-none"
        >
            <div class="flex-1"></div>

            <!-- Right Side: Floating Cover Art -->
            <div
                class="shrink-0 hidden lg:block pointer-events-auto"
                style="width: {imageWidth}px;"
                on:contextmenu={handleContextMenu}
            >
                <div
                    class="rounded-lg overflow-hidden shadow-2xl border border-border-primary transform transition-transform hover:scale-105 duration-300 cursor-pointer"
                    on:click={() => (showFullscreenImage = true)}
                    on:keydown={(e) =>
                        e.key === "Enter" && (showFullscreenImage = true)}
                    role="button"
                    tabindex="0"
                >
                    <img
                        src={bgImage}
                        alt="Cover"
                        class="w-full h-auto"
                        on:error={handleImageError}
                    />
                </div>
            </div>
        </div>

        <!-- Bottom: Title Section -->
        <div
            class="pointer-events-none transition-transform duration-300 ease-out"
            style="transform: translateY(-{offset}px);"
        >
            {#if element.playStatus === 2}
                <div
                    class="inline-block px-3 py-1 rounded-full bg-accent-success/30 text-accent-success text-sm font-bold mb-4 border border-accent-success/100"
                >
                    クリア済み
                </div>
            {/if}
            <h1
                class="{element.gamename.length > 40
                    ? 'text-3xl'
                    : element.gamename.length > 20
                      ? 'text-4xl'
                      : 'text-5xl'} font-bold text-white leading-tight drop-shadow-lg max-w-4xl"
            >
                {element.gamename}
            </h1>
        </div>
    </div>

    {#if menu.isOpen}
        <ContextMenu
            x={menu.x}
            y={menu.y}
            options={menuOptions}
            on:close={() => (menu.isOpen = false)}
        />
    {/if}

    <!-- Fullscreen Image Viewer -->
    {#if showFullscreenImage}
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-sm pointer-events-auto"
            on:click={() => (showFullscreenImage = false)}
            on:keydown={(e) =>
                (e.key === "Escape" || e.key === "Enter") &&
                (showFullscreenImage = false)}
            role="button"
            tabindex="0"
        >
            <div
                class="flex flex-col items-center max-w-[90vw] max-h-[90vh] w-full h-[80vh]"
            >
                <ZoomableImage
                    src={bgImage}
                    alt={element.gamename}
                    class="max-w-full max-h-full shadow-2xl rounded-lg"
                />
            </div>
        </div>
    {/if}
</div>
