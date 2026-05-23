<script lang="ts">
  import WorkLayout from "@/components/Work/WorkLayout.svelte";
  import type { Work } from "@/lib/types";
  import { onMount } from "svelte";
  import { backgroundState } from "@/store/background";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import SimpleBar from "simplebar";

  export let work: Work;
  export let element: CollectionElement;

  onMount(() => {
    // Background is now handled by Hero.svelte internally
  });
  let scrollY = 0;

  const simplebar = (node: HTMLElement) => {
    const instance = new SimpleBar(node, {
      scrollbarMinSize: 64,
      autoHide: false,
    });
    const scrollElement = instance.getScrollElement();

    const handleScroll = () => {
      scrollY = scrollElement?.scrollTop ?? 0;
    };

    scrollElement?.addEventListener("scroll", handleScroll);

    return {
      destroy() {
        scrollElement?.removeEventListener("scroll", handleScroll);
        instance.unMount();
      },
    };
  };
</script>

<div
  use:simplebar
  class="work-detail-page h-full w-full overflow-x-hidden overflow-y-auto"
>
  <div class="w-full min-h-0 flex justify-center">
    {#key work.imgUrl}
      <WorkLayout {work} {element} {scrollY} />
    {/key}
  </div>
</div>

<style lang="scss">
  :global(.work-detail-page .simplebar-track.simplebar-vertical) {
    right: 0;
    width: 0.55rem;
    pointer-events: auto;
  }

  :global(.work-detail-page .simplebar-scrollbar::before) {
    left: 1px;
    right: 1px;
  }
</style>
