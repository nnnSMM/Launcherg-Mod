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
    export let scrollY: number = 0;
    export let offset: number = 0;
    export let actualRenderedH: number = 0;
    export let heroHeight: number = 0;

    let innerHeight = 0;
    $: defaultNegativeMargin = Math.min(Math.max(48, innerHeight * 0.08), 96);
    $: glassTopY = heroHeight - defaultNegativeMargin - offset;

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

    let loadedImage: HTMLImageElement | null = null;
    let isImageLoading = false;

    // 画像を事前にロードしてキャッシュしておく
    $: if (bgImage) {
        preloadImage(bgImage);
    }

    async function preloadImage(src: string) {
        isImageLoading = true;
        const img = new Image();
        img.src = src;
        try {
            await new Promise<void>((resolve, reject) => {
                img.onload = () => resolve();
                img.onerror = () => reject();
            });
            loadedImage = img;
            renderBgToCanvas();
        } catch (e) {
            console.error("Failed to preload hero image:", e);
        } finally {
            isImageLoading = false;
        }
    }

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
                    // サムネイル変更後、画像の再フェッチを促すためupdatedAtを更新
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

    // Canvasで「下だけ引き伸ばし」を実現
    import { onMount } from "svelte";
    let canvasEl: HTMLCanvasElement;
    let containerEl: HTMLDivElement;

    function renderBgToCanvas() {
        if (!canvasEl || !containerEl || !loadedImage) return;

        const rect = containerEl.getBoundingClientRect();
        const w = Math.round(rect.width);
        const h = Math.round(rect.height);
        if (!w || !h) return;

        canvasEl.width = w;
        canvasEl.height = h;

        const ctx = canvasEl.getContext('2d');
        if (!ctx) return;

        const img = loadedImage;
        const scale = w / img.naturalWidth;
        const renderedH = Math.round(img.naturalHeight * scale);

        // 実際の画像の高さを親（WorkLayout）に渡す
        if (Math.abs(actualRenderedH - renderedH) > 0.5) {
            actualRenderedH = renderedH;
        }

        // 画像を全幅で描画
        ctx.filter = 'none';
        ctx.drawImage(img, 0, 0, w, renderedH);

        // コンテナが画像より高い場合、伸ばした画像を3x3ミラータイルでシームレスにしてからにじみを適用
        if (h > renderedH) {
            const flipH = h - renderedH;
            const iw = img.naturalWidth;
            const ih = img.naturalHeight;
            const srcStripRows = Math.max(1, Math.ceil(ih * 0.05));

            // Step 0: 伸ばした部分をオフスクリーンに描画（これがタイルの単位）
            const off0 = document.createElement('canvas');
            off0.width = w;
            off0.height = flipH;
            const off0Ctx = off0.getContext('2d');
            if (!off0Ctx) return;
            off0Ctx.drawImage(img, 0, ih - srcStripRows, iw, srcStripRows, 0, 0, w, flipH);

            // Step 1: off0を使って3x3ミラータイルを作成（中央=オリジナル、外側=反転）
            const off1 = document.createElement('canvas');
            off1.width = w * 3;
            off1.height = flipH * 3;
            const off1Ctx = off1.getContext('2d');
            if (!off1Ctx) return;

            for (let row = 0; row < 3; row++) {
                for (let col = 0; col < 3; col++) {
                    const flipX = col !== 1; // 外側の列は水平反転
                    const flipY = row !== 1; // 外側の行は垂直反転（中央行はそのまま）
                    off1Ctx.save();
                    off1Ctx.translate(
                        col * w + (flipX ? w : 0),
                        row * flipH + (flipY ? flipH : 0)
                    );
                    off1Ctx.scale(flipX ? -1 : 1, flipY ? -1 : 1);
                    off1Ctx.drawImage(off0, 0, 0);
                    off1Ctx.restore();
                }
            }

            // Step 2: 3x3全体にインクにじみフィルターを適用
            const off2 = document.createElement('canvas');
            off2.width = w * 3;
            off2.height = flipH * 3;
            const off2Ctx = off2.getContext('2d');
            if (!off2Ctx) return;
            off2Ctx.filter = 'url(#ink-water)';
            off2Ctx.drawImage(off1, 0, 0);
            off2Ctx.filter = 'none';

            // Step 3: 中央タイル（w × flipH）だけをメインキャンバスに転写
            ctx.drawImage(
                off2,
                w, flipH,       // 中央タイルの開始座標
                w, flipH,       // 中央タイルのサイズ
                0, renderedH,   // メインキャンバスの配置先
                w, flipH
            );
        }
    }

    onMount(() => {
        const observer = new ResizeObserver(() => renderBgToCanvas());
        if (containerEl) observer.observe(containerEl);
        renderBgToCanvas();
        return () => observer.disconnect();
    });

    $: bgImage, (() => { if (canvasEl) renderBgToCanvas(); })();
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div 
    bind:clientWidth={heroWidth}
    bind:clientHeight={heroHeight}
    class="relative w-full min-h-[60vh] min-h-[300px] group flex flex-col pointer-events-none"
>
    <!-- Background Image -->
    <!-- Background Image with Parallax -->
    <!-- SVGフィルター定義: インクのにじみ効果（静的） -->
    <svg style="position: absolute; width: 0; height: 0; overflow: hidden;">
        <defs>
            <filter id="ink-water" x="-50%" y="-50%" width="200%" height="200%" color-interpolation-filters="sRGB">
                <feTurbulence
                    type="turbulence"
                    baseFrequency="0.008 0.006"
                    numOctaves="4"
                    seed="8"
                    result="turbulence"
                />
                <feDisplacementMap
                    in="SourceGraphic"
                    in2="turbulence"
                    scale="180"
                    xChannelSelector="R"
                    yChannelSelector="G"
                    result="displaced"
                />
                <!-- 変位後にブラーをかけて隣接ピクセルを混合・平均化 -->
                <feGaussianBlur in="displaced" stdDeviation="40" />
            </filter>
        </defs>
    </svg>

    <div class="absolute top-0 left-0 right-0 h-[calc(100%+800px)] z-0 overflow-hidden pointer-events-none">
        <div
            bind:this={containerEl}
            class="w-full h-full overflow-hidden"
            style="transform: translateY({scrollY *
                0.65}px); will-change: transform;"
        >
            <canvas
                bind:this={canvasEl}
                class="blur-[2px] opacity-100"
                style="display: block; width: 100%; height: 100%;"
            />
            <!-- 背景色オーバーレイ: 画像端とGlass端の「より高い方」から開始して、可読性を確保 -->
            <div
                class="absolute left-0 right-0 bg-bg-primary/55"
                style="top: {Math.min(actualRenderedH, glassTopY) + 20}px; bottom: 0;"
            />
        </div>
        <!-- グラデーション (下からフェードアウト) - 範囲を広げてより緩やかに -->
        <div
            class="absolute bottom-0 left-0 right-0 h-[95%] bg-gradient-to-t from-bg-primary via-bg-primary/0 to-transparent pointer-events-none"
        />
    </div>

    <!-- Content -->
    <div class="relative z-10 flex-1 flex flex-col justify-between pt-12 px-12 pb-24 pointer-events-none">
        <!-- Top: Title Section -->
        <div class="flex items-start justify-between gap-12 pointer-events-none">
            <div class="flex-1"></div>

            <!-- Right Side: Floating Cover Art -->
            <div
                class="shrink-0 hidden lg:block pointer-events-auto"
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
