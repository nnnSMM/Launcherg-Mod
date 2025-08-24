<script lang="ts">
  import WorkLayout from "@/components/Work/WorkLayout.svelte";
  import type { Work } from "@/lib/types";
  import { onMount, onDestroy } from "svelte";
  import { backgroundState } from "@/store/background";

  export let work: Work;
  let scrollY = 0;
  let scrollContainer: HTMLDivElement;

  const handleScroll = () => {
    if (scrollContainer) {
      scrollY = scrollContainer.scrollTop;
    }
  };

  onMount(() => {
    backgroundState.set({
      imageUrl: work.imgUrl,
      opacity: 0.2,
    });
  });

  onDestroy(() => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });
  });
</script>

<div
  class="h-full w-full overflow-x-hidden overflow-y-auto"
  bind:this={scrollContainer}
  on:scroll={handleScroll}
>
  <div class="w-full min-h-0 flex justify-center">
    {#key work.imgUrl}
      <WorkLayout {work} {scrollY} />
    {/key}
  </div>
</div>
