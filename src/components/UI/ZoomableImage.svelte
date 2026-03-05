<script lang="ts">
    import { createEventDispatcher, tick } from "svelte";

    export let src: string;
    export let alt: string = "";
    let className = "";
    export { className as class };

    export let scale = 1;
    export let isZoomed = false; // Expose zoom state relative to "fit"

    let containerWidth = 0;
    let containerHeight = 0;
    let naturalWidth = 0;
    let naturalHeight = 0;

    let translateX = 0;
    let translateY = 0;

    let isDragging = false;
    let startX = 0;
    let startY = 0;
    let wasDragged = false;
    let isImagePositioned = false;
    let hasInitialLayout = false;

    // Limits
    let minScale = 0.1;
    let maxScale = 8.0;
    let fitScale = 1.0;

    let imgElement: HTMLImageElement;
    let displayedSrc = "";
    let activeSrc = "";
    let srcRequestId = 0;

    const dispatch = createEventDispatcher();

    const resetInteractiveState = () => {
        naturalWidth = 0;
        naturalHeight = 0;
        scale = 1;
        translateX = 0;
        translateY = 0;
        isZoomed = false;
        isDragging = false;
        wasDragged = false;
        isImagePositioned = false;
        hasInitialLayout = false;
    };

    const applyInitialLayout = (width: number, height: number): boolean => {
        if (!width || !height || !containerWidth || !containerHeight) {
            return false;
        }

        naturalWidth = width;
        naturalHeight = height;

        const scaleX = containerWidth / width;
        const scaleY = containerHeight / height;
        fitScale = Math.min(scaleX, scaleY);
        minScale = fitScale;
        scale = fitScale;
        centerImage();

        isImagePositioned = true;
        hasInitialLayout = true;
        void tick().then(() => {
            dispatch("ready");
        });

        return true;
    };

    const swapDisplayedSource = (
        nextSrc: string,
        width: number,
        height: number,
    ) => {
        resetInteractiveState();
        const hasLayout = applyInitialLayout(width, height);
        displayedSrc = nextSrc;

        if (!hasLayout) {
            naturalWidth = width;
            naturalHeight = height;
        }
    };

    const preloadAndSwap = async (nextSrc: string, requestId: number) => {
        const preloadImage = new Image();
        const { width, height } = await new Promise<{
            width: number;
            height: number;
        }>((resolve) => {
            const cleanup = () => {
                preloadImage.onload = null;
                preloadImage.onerror = null;
            };

            preloadImage.onload = () => {
                cleanup();
                resolve({
                    width: preloadImage.naturalWidth,
                    height: preloadImage.naturalHeight,
                });
            };

            preloadImage.onerror = () => {
                cleanup();
                resolve({ width: 0, height: 0 });
            };

            preloadImage.src = nextSrc;
        });

        if (requestId !== srcRequestId || activeSrc !== nextSrc) return;
        swapDisplayedSource(nextSrc, width, height);
    };

    $: if (!src) {
        activeSrc = "";
        displayedSrc = "";
        resetInteractiveState();
    } else if (src !== activeSrc) {
        activeSrc = src;
        const requestId = ++srcRequestId;
        if (!displayedSrc) {
            swapDisplayedSource(src, 0, 0);
        } else {
            void preloadAndSwap(src, requestId);
        }
    }

    function resetZoom() {
        if (!naturalWidth || !containerWidth || !containerHeight) return;

        // Calculate fit scale
        const scaleX = containerWidth / naturalWidth;
        const scaleY = containerHeight / naturalHeight;
        fitScale = Math.min(scaleX, scaleY);

        minScale = fitScale;
        scale = fitScale;

        centerImage();
        isZoomed = false;
    }

    function centerImage() {
        if (!naturalWidth || !containerWidth || !containerHeight) return;
        translateX = (containerWidth - naturalWidth * scale) / 2;
        translateY = (containerHeight - naturalHeight * scale) / 2;
    }

    function onImageLoad() {
        if (imgElement) {
            naturalWidth = imgElement.naturalWidth;
            naturalHeight = imgElement.naturalHeight;
        }
    }

    // Wait until both image and container are measurable, then show fitted image.
    $: if (
        naturalWidth &&
        naturalHeight &&
        containerWidth &&
        containerHeight &&
        !hasInitialLayout
    ) {
        resetZoom();
        isImagePositioned = true;
        hasInitialLayout = true;

        tick().then(() => {
            dispatch("ready");
        });
    }

    // Reactive update for resize
    $: if (containerWidth && containerHeight && naturalWidth) {
        // Recalculate fit
        const scaleX = containerWidth / naturalWidth;
        const scaleY = containerHeight / naturalHeight;
        const newFit = Math.min(scaleX, scaleY);

        // If we were at the old fit scale (approx), simplify update to new fit
        // Avoid cycle by not reading isZoomed reactive var here, but calculating logic locally or just checking scale
        if (Math.abs(scale - fitScale) < 0.001) {
            scale = newFit;
            centerImage();
        }

        fitScale = newFit;
        minScale = newFit;
    }

    // Update isZoomed
    $: isZoomed = scale > fitScale * 1.001;

    function handleWheel(e: WheelEvent) {
        e.preventDefault();
        if (!naturalWidth) return;

        const ZOOM_FACTOR = 0.15;
        const direction = -Math.sign(e.deltaY);
        const multiplier = 1 + direction * ZOOM_FACTOR;

        const oldScale = scale;
        // Clamp
        let newScale = scale * multiplier;
        newScale = Math.min(Math.max(minScale, newScale), maxScale);

        if (newScale <= minScale * 1.01) {
            newScale = minScale;
            scale = minScale;
            centerImage();
            return;
        }

        // Logic: newTranslate = mouse - (mouse - oldTranslate) * (newScale / oldScale)
        // Mouse relative to container
        const target = e.currentTarget as HTMLElement;
        const rect = target.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        // Apply zoom
        const ratio = newScale / oldScale;
        translateX = mouseX - (mouseX - translateX) * ratio;
        translateY = mouseY - (mouseY - translateY) * ratio;

        scale = newScale;
    }

    function handleMouseDown(e: MouseEvent) {
        if (e.button !== 0) return;
        // Allow drag if scaled > minScale OR if we want to allow panning even at fit if image > container?
        // But logic says fit = contain, so image <= container always.
        // So only drag if zoomed.
        if (isZoomed) {
            isDragging = true;
            startX = e.clientX - translateX;
            startY = e.clientY - translateY;
            wasDragged = false;
            e.preventDefault();
        }
    }

    function handleMouseMove(e: MouseEvent) {
        if (isDragging) {
            translateX = e.clientX - startX;
            translateY = e.clientY - startY;
            wasDragged = true;
            e.preventDefault();
        }
    }

    function handleMouseUp() {
        isDragging = false;
    }

    function handleClick(e: MouseEvent) {
        if (wasDragged) {
            e.stopImmediatePropagation();
            e.stopPropagation();
            wasDragged = false;
        } else {
            dispatch("click", e);
        }
    }
</script>

<svelte:window on:mouseup={handleMouseUp} on:mousemove={handleMouseMove} />

<div
    class="w-full h-full overflow-hidden relative select-none {className}"
    bind:clientWidth={containerWidth}
    bind:clientHeight={containerHeight}
    on:wheel={handleWheel}
    on:mousedown={handleMouseDown}
    role="presentation"
>
    <!-- 
        img is absolute. 
        transform-origin top-left (0 0) to make math simple.
        width/height not set (auto) -> native size.
    -->
    <img
        bind:this={imgElement}
        src={displayedSrc}
        {alt}
        on:load={onImageLoad}
        class="absolute top-0 left-0 max-w-none max-h-none origin-top-left"
        class:invisible={!isImagePositioned}
        class:cursor-grab={isZoomed && !isDragging}
        class:cursor-grabbing={isDragging}
        style="transform: translate({translateX}px, {translateY}px) scale({scale})"
        on:click={handleClick}
        on:keydown={() => {}}
        draggable="false"
    />
</div>
