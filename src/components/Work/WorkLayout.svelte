<script lang="ts">
  import Hero from "@/components/Work/Hero.svelte";
  import GlassInfo from "@/components/Work/GlassInfo.svelte";
  import type { Work, CollectionElement } from "@/lib/types";

  export let work: Work;
  export let element: CollectionElement;
  export let scrollY: number = 0;
</script>

<div class="w-full min-h-full bg-bg-primary">
  <Hero {element} {scrollY} />
  <div class="w-full max-w-[1600px] mx-auto">
    <GlassInfo {work} {element} />
  </div>
</div>
