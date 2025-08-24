<script lang="ts">
  import WorkLayout from "@/components/Work/WorkLayout.svelte";
  import type { Work } from "@/lib/types";

  export let work: Work;
  let scrollY = 0;
  let scrollContainer: HTMLDivElement;

  const handleScroll = () => {
    if (scrollContainer) {
      scrollY = scrollContainer.scrollTop;
    }
  };
</script>

<div
  class="fixed top-0 left-0 w-screen h-screen z-[-1] bg-cover bg-center backdrop-blur-xl op-20"
  style="background-image: url({work.imgUrl})"
/>
<div
  class="h-full w-full overflow-x-hidden overflow-y-auto bg-bg-primary"
  bind:this={scrollContainer}
  on:scroll={handleScroll}
>
  <div class="w-full min-h-0 flex justify-center">
    {#key work.imgUrl}
      <WorkLayout {work} {scrollY} />
    {/key}
  </div>
</div>
