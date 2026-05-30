<script lang="ts">
  import Hero from "@/components/Work/Hero.svelte";
  import GlassInfo from "@/components/Work/GlassInfo.svelte";
  import type { Work, CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";

  export let work: Work;
  export let element: CollectionElement;
  export let scrollY: number = 0;

  let heroHeight = 0;
  let innerHeight = 0;

  // Glass が Hero にどれだけ食い込むか（Hero.svelte と同期）
  $: defaultNegativeMargin = Math.min(Math.max(48, innerHeight * 0.08), 96);
  $: glassTopY = heroHeight - defaultNegativeMargin;

  $: bgImage =
    element.thumbnail && element.thumbnail.trim() !== ""
      ? `${convertFileSrc(element.thumbnail)}?v=${element.updatedAt}`
      : "/images/dummy_thumbnail.svg";
</script>

<svelte:window bind:innerHeight />

<div class="w-full min-h-full relative bg-transparent">
  <!-- 背景画像レイヤー: 黒いオーバーレイやにじみを取り除き、画像のみをぼかして配置 -->
  {#if bgImage}
    <div 
      class="absolute top-0 left-0 right-0 overflow-hidden pointer-events-none z-0"
      style="height: {heroHeight}px; transform: translateY({scrollY * 0.65}px); will-change: transform;"
    >
      <img
        src={bgImage}
        alt="Background"
        class="w-full h-full object-cover blur-[2px] opacity-90"
      />
    </div>
  {/if}

  <Hero 
    {element} 
    offset={0}
    glassBoundaryY={glassTopY}
    bind:heroHeight
  />

  <div 
    class="relative w-full max-w-[1600px] mx-auto transition-transform duration-300 ease-out z-10"
    style="margin-top: -{defaultNegativeMargin}px;"
  >
    <GlassInfo {work} {element} />
  </div>
</div>
