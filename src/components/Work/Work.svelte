<script lang="ts">
  import WorkLayout from "@/components/Work/WorkLayout.svelte";
  import type { Work } from "@/lib/types";
  import { onMount } from "svelte";
  import { backgroundState } from "@/store/background";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";

  export let work: Work;
  export let element: CollectionElement;

  onMount(() => {
    // Background is now handled by Hero.svelte internally
  });
  let scrollY = 0;

  const handleScroll = (e: UIEvent) => {
    const target = e.target as HTMLElement;
    scrollY = target.scrollTop;
  };
</script>

<div
  class="work-detail-page h-full w-full overflow-x-hidden overflow-y-auto"
  on:scroll={handleScroll}
>
  <div class="w-full min-h-0 flex justify-center">
    {#key work.imgUrl}
      <WorkLayout {work} {element} {scrollY} />
    {/key}
  </div>
</div>
