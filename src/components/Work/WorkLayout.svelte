<script lang="ts">
  import Hero from "@/components/Work/Hero.svelte";
  import GlassInfo from "@/components/Work/GlassInfo.svelte";
  import type { Work, CollectionElement } from "@/lib/types";

  export let work: Work;
  export let element: CollectionElement;
  export let scrollY: number = 0;

  let actualRenderedH = 0;
  let heroHeight = 0;
  let innerHeight = 0;

  // Glass が Hero にどれだけ食い込むか（Hero.svelte と同期）
  $: defaultNegativeMargin = Math.min(Math.max(48, innerHeight * 0.08), 96);

  // 画像が Hero コンテナより短い場合に、コンテンツを持ち上げるためのオフセットを計算
  $: offset = Math.max(0, heroHeight - actualRenderedH);

  // 下限（画像の下端）を守りつつ、40px 下に下げるための有効オフセット
  $: effectiveOffset = Math.max(0, offset - 40);
</script>

<svelte:window bind:innerHeight />

<div class="w-full min-h-full bg-bg-primary">
  <Hero 
    {element} 
    {scrollY} 
    offset={effectiveOffset} 
    bind:actualRenderedH 
    bind:heroHeight
  />
  <div 
    class="w-full max-w-[1600px] mx-auto transition-transform duration-300 ease-out"
    style="margin-top: -{effectiveOffset + defaultNegativeMargin}px;"
  >
    <GlassInfo {work} {element} />
  </div>
</div>
