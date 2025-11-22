<script lang="ts">
  import Hero from "@/components/Work/Hero.svelte";
  import GlassInfo from "@/components/Work/GlassInfo.svelte";
  import type { Work, CollectionElement } from "@/lib/types";

  import ScreenshotGallery from "@/components/Work/ScreenshotGallery.svelte";

  export let work: Work;
  export let element: CollectionElement;
  export let scrollY: number = 0;
</script>

<div class="w-full min-h-full bg-bg-primary">
  <Hero {work} {element} {scrollY} />
  <GlassInfo {work} {element} />
  <ScreenshotGallery gameId={work.id} />
</div>
