<script lang="ts">
  import { onMount } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Hero from "@/components/Work/Hero.svelte";
  import GlassInfo from "@/components/Work/GlassInfo.svelte";
  import type { Work, CollectionElement } from "@/lib/types";

  export let work: Work;
  export let element: CollectionElement;
  export let scrollY: number = 0;

  let actualRenderedH = 0;
  let heroHeight = 0;
  let innerHeight = 0;
  let layoutWidth = 0;
  let layoutHeight = 0;

  // Glass が Hero にどれだけ食い込むか（Hero.svelte と同期）
  $: defaultNegativeMargin = Math.min(Math.max(48, innerHeight * 0.08), 96);

  // 画像が Hero コンテナより短い場合に、コンテンツを持ち上げるためのオフセットを計算
  $: offset = Math.max(0, heroHeight - actualRenderedH);

  // 下限（画像の下端）を守りつつ、40px 下に下げるための有効オフセット
  $: effectiveOffset = Math.max(0, offset - 40);

  // 背景描画ロジックの移行
  $: bgImage =
    element.thumbnail && element.thumbnail.trim() !== ""
      ? `${convertFileSrc(element.thumbnail)}?v=${element.updatedAt}`
      : "/images/dummy_thumbnail.svg";

  let loadedImage: HTMLImageElement | null = null;
  $: if (bgImage) {
    preloadImage(bgImage);
  }

  async function preloadImage(src: string) {
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
      console.error("Failed to preload layout background image:", e);
    }
  }

  let canvasEl: HTMLCanvasElement;
  let containerEl: HTMLDivElement;

  function renderBgToCanvas() {
    if (!canvasEl || !containerEl || !loadedImage) return;

    const w = layoutWidth;
    const h = layoutHeight;
    if (!w || !h) return;

    canvasEl.width = w;
    canvasEl.height = h;

    const ctx = canvasEl.getContext("2d");
    if (!ctx) return;

    const img = loadedImage;
    const scale = w / img.naturalWidth;
    const renderedH = Math.round(img.naturalHeight * scale);

    // 実際の画像の高さを更新
    if (Math.abs(actualRenderedH - renderedH) > 0.5) {
      actualRenderedH = renderedH;
    }

    // ガラスの上端位置（Hero 内の計算を模倣）
    const glassTopY = heroHeight - defaultNegativeMargin - effectiveOffset;
    const bleedStartY = Math.min(renderedH, glassTopY + 50);

    ctx.filter = "none";
    const sourceHeight = bleedStartY / scale;
    ctx.drawImage(
      img,
      0, 0, img.naturalWidth, sourceHeight,
      0, 0, w, bleedStartY
    );

    if (h > bleedStartY) {
      const tileH = bleedStartY;
      const iw = img.naturalWidth;

      const off0 = document.createElement("canvas");
      off0.width = w;
      off0.height = tileH;
      const off0Ctx = off0.getContext("2d");
      if (!off0Ctx) return;
      off0Ctx.drawImage(img, 0, 0, iw, sourceHeight, 0, 0, w, tileH);

      const off1 = document.createElement("canvas");
      off1.width = w * 3;
      off1.height = tileH * 3;
      const off1Ctx = off1.getContext("2d");
      if (!off1Ctx) return;

      for (let row = 0; row < 3; row++) {
        for (let col = 0; col < 3; col++) {
          const flipX = col !== 1;
          const flipY = row !== 1;
          off1Ctx.save();
          off1Ctx.translate(col * w + (flipX ? w : 0), row * tileH + (flipY ? tileH : 0));
          off1Ctx.scale(flipX ? -1 : 1, flipY ? -1 : 1);
          off1Ctx.drawImage(off0, 0, 0);
          off1Ctx.restore();
        }
      }

      const off2 = document.createElement("canvas");
      off2.width = w * 3;
      off2.height = tileH * 3;
      const off2Ctx = off2.getContext("2d");
      if (!off2Ctx) return;
      off2Ctx.filter = "url(#ink-water)";
      off2Ctx.drawImage(off1, 0, 0);

      // 反転部分をページ最下部まで「伸ばして」描画
      ctx.drawImage(
        off2,
        w, 2 * tileH,
        w, tileH,
        0, bleedStartY,
        w, h - bleedStartY
      );
    }
  }

  $: glassTopY = heroHeight - defaultNegativeMargin - effectiveOffset;

  onMount(() => {
    const observer = new ResizeObserver(() => renderBgToCanvas());
    if (containerEl) observer.observe(containerEl);
    renderBgToCanvas();
    return () => observer.disconnect();
  });

  $: if (canvasEl && (layoutWidth || layoutHeight || heroHeight)) {
    renderBgToCanvas();
  }
</script>

<svelte:window bind:innerHeight />

<div 
  class="w-full min-h-full relative bg-bg-primary" 
  bind:clientWidth={layoutWidth}
  bind:clientHeight={layoutHeight}
>
  <!-- 背景描画レイヤー（Hero 内から移行） -->
  <div class="absolute inset-0 z-0 overflow-hidden pointer-events-none">
    <!-- SVGフィルター定義 -->
    <svg style="position: absolute; width: 0; height: 0; overflow: hidden;">
      <defs>
        <filter id="ink-water" x="-50%" y="-50%" width="200%" height="200%" color-interpolation-filters="sRGB">
          <feTurbulence type="turbulence" baseFrequency="0.008 0.006" numOctaves="4" seed="8" result="turbulence" />
          <feDisplacementMap in="SourceGraphic" in2="turbulence" scale="120" xChannelSelector="R" yChannelSelector="G" result="displaced" />
          <feGaussianBlur in="displaced" stdDeviation="20" />
        </filter>
      </defs>
    </svg>

    <div
      bind:this={containerEl}
      class="w-full h-full overflow-hidden"
      style="transform: translateY({scrollY * 0.65}px); will-change: transform;"
    >
      <canvas
        bind:this={canvasEl}
        class="blur-[2px] opacity-100"
        style="display: block; width: 100%; height: 100%;"
      />
      <!-- 背景色オーバーレイ: 曇りグラス効果（画像端とGlass端の「より高い方」から開始） -->
      <div
        class="absolute left-0 right-0 bg-bg-primary/40 border-t border-border-primary"
        style="top: {Math.min(actualRenderedH, glassTopY) + 30}px; bottom: 0; backdrop-filter: blur(24px);"
      />
    </div>
  </div>

  <!-- Hero背景からページ背景色へ繋ぐグラデーション（下部を不透明化して白い露出を防ぐ） -->
  <div 
    class="absolute inset-0 pointer-events-none z-5"
    style="background: linear-gradient(to bottom, transparent 0%, transparent 40vh, rgba(var(--color-bg-primary), 0.65) 100vh, rgba(var(--color-bg-primary), 0.65) 100%);"
  />

  <Hero 
    {element} 
    offset={effectiveOffset}
    bind:heroHeight
  />

  <div 
    class="relative w-full max-w-[1600px] mx-auto transition-transform duration-300 ease-out z-10"
    style="margin-top: -{effectiveOffset + defaultNegativeMargin}px;"
  >
    <GlassInfo {work} {element} />
  </div>
</div>
