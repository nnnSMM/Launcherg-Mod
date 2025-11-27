<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { CollectionElement } from "@/lib/types";
    import { createEventDispatcher } from "svelte";
    import { commandPlayGame } from "@/lib/command";

    import { open } from "@tauri-apps/plugin-dialog";
    import { commandUpdateGameImage } from "@/lib/command";
    import ContextMenu from "@/components/UI/ContextMenu.svelte";

    export let element: CollectionElement;
    export let scrollY: number = 0;

    const dispatch = createEventDispatcher();

    $: bgImage =
        element.thumbnail && element.thumbnail.trim() !== ""
            ? `${convertFileSrc(element.thumbnail)}?v=${element.updatedAt}`
            : "/images/dummy_thumbnail.svg";

    const handleImageError = (e: Event) => {
        const img = e.target as HTMLImageElement;
        img.src = "/images/dummy_thumbnail.svg";
    };

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
                    window.location.reload();
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

    // Target area for the cover image (width * height)
    // Adjust this value to change the overall size of the image while maintaining area consistency
    $: TARGET_AREA = innerWidth < 1280 ? 90000 : 130000;

    $: imageWidth = (() => {
        const width = element.thumbnailWidth || 16;
        const height = element.thumbnailHeight || 9;
        const ratio = width / height;
        // Width = Sqrt(Area * Ratio)
        return Math.sqrt(TARGET_AREA * ratio);
    })();
</script>

<svelte:window bind:innerWidth />

<div class="relative w-full min-h-[60vh] min-h-[300px] group flex flex-col">
    <!-- Background Image -->
    <!-- Background Image with Parallax -->
    <div class="absolute inset-0 z-0 overflow-hidden rounded-b-xl">
        <div
            class="w-full h-[100%] -mt"
            style="transform: translateY({scrollY *
                0.65}px); will-change: transform;"
        >
            <img
                src={bgImage}
                alt={element.gamename}
                class="w-full h-full object-cover object-top opacity-100 blur-sm"
                on:error={handleImageError}
            />
        </div>
        <div
            class="absolute inset-0 bg-gradient-to-t from-bg-primary/20 via-bg-primary/0 to-transparent"
        />
    </div>

    <!-- Content -->
    <div class="relative z-10 flex-1 flex flex-col justify-between p-12">
        <!-- Top: Title Section -->
        <div class="flex items-start justify-between gap-12">
            <div class="flex-1"></div>

            <!-- Right Side: Floating Cover Art -->
            <div
                class="shrink-0 hidden lg:block"
                style="width: {imageWidth}px;"
                on:contextmenu={handleContextMenu}
            >
                <div
                    class="rounded-lg overflow-hidden shadow-2xl border border-white/10 transform transition-transform hover:scale-105 duration-300 cursor-pointer"
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
        <div>
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
            class="fixed inset-0 z-50 bg-black/95 flex items-center justify-center p-8 cursor-pointer"
            on:click={() => (showFullscreenImage = false)}
            on:keydown={(e) =>
                (e.key === "Escape" || e.key === "Enter") &&
                (showFullscreenImage = false)}
            role="button"
            tabindex="0"
        >
            <img
                src={bgImage}
                alt={element.gamename}
                class="w-auto max-h-screen object-contain"
            />
        </div>
    {/if}
</div>
