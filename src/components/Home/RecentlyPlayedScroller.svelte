<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import SimpleBar from "simplebar";

  export const scrollBy = (options: ScrollToOptions) => {
    scrollEl?.scrollBy(options);
  };

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>();

  let scrollEl: HTMLElement | null = null;

  const simplebarAction = (node: HTMLElement) => {
    const simplebarInstance = new SimpleBar(node, {});
    scrollEl = simplebarInstance.getScrollElement();

    const onScroll = (e: Event) => {
      dispatcher("scroll", { event: e });
    };
    scrollEl?.addEventListener("scroll", onScroll);

    return {
      destroy: () => {
        scrollEl?.removeEventListener("scroll", onScroll);
      },
    };
  };

  let isDown = false;
  let startX: number;
  let scrollLeft: number;

  const handleMouseDown = (e: MouseEvent) => {
    if (!scrollEl) return;
    isDown = true;
    scrollEl.classList.add("active");
    startX = e.pageX - scrollEl.offsetLeft;
    scrollLeft = scrollEl.scrollLeft;
  };

  const handleMouseLeave = () => {
    if (!scrollEl) return;
    isDown = false;
    scrollEl.classList.remove("active");
  };

  const handleMouseUp = () => {
    if (!scrollEl) return;
    isDown = false;
    scrollEl.classList.remove("active");
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDown || !scrollEl) return;
    e.preventDefault();
    const x = e.pageX - scrollEl.offsetLeft;
    const walk = (x - startX) * 2; //scroll-fast
    scrollEl.scrollLeft = scrollLeft - walk;
  };
</script>

<!-- Add a style block to control SimpleBar's appearance -->
<style>
  .scroller {
    cursor: grab;
    overscroll-behavior-y: contain;
  }
  .scroller.active {
    cursor: grabbing;
  }
  :global(.recently-played-scroller .simplebar-track.simplebar-vertical) {
    display: none !important;
  }
  :global(.recently-played-scroller .simplebar-scrollbar::before) {
    background-color: rgba(160, 160, 160, 0.8);
  }
</style>

<div
  use:simplebarAction
  class="overflow-x-auto recently-played-scroller scroller"
  on:mousedown={handleMouseDown}
  on:mouseleave={handleMouseLeave}
  on:mouseup={handleMouseUp}
  on:mousemove={handleMouseMove}
>
  <slot />
</div>
