<script lang="ts">
  import { createEventDispatcher, onDestroy } from "svelte";
  import SimpleBar from "simplebar";

  export const scrollBy = (options: ScrollToOptions) => {
    scrollEl?.scrollBy(options);
  };

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>();

  let scrollEl: HTMLElement | null = null;
  let scrollerDiv: HTMLDivElement;

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
    scrollerDiv.classList.add("active");
    startX = e.pageX - scrollEl.offsetLeft;
    scrollLeft = scrollEl.scrollLeft;

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  };

  const handleMouseUp = () => {
    isDown = false;
    scrollerDiv.classList.remove("active");

    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDown || !scrollEl) return;
    e.preventDefault();
    const x = e.pageX - scrollEl.offsetLeft;
    const walk = (x - startX) * 2; //scroll-fast
    scrollEl.scrollLeft = scrollLeft - walk;
  };

  onDestroy(() => {
    // Clean up global listeners if component is destroyed while dragging
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);
  });
</script>

<!-- Add a style block to control SimpleBar's appearance -->
<style>
  .scroller {
    cursor: grab;
    user-select: none; /* Prevent text selection during drag */
  }
  .scroller.active {
    cursor: grabbing;
  }
  :global(.recently-played-scroller .simplebar-content-wrapper) {
    overscroll-behavior-y: contain;
  }
  :global(.recently-played-scroller .simplebar-track.simplebar-vertical) {
    display: none !important;
  }
  :global(.recently-played-scroller .simplebar-scrollbar::before) {
    background-color: rgba(160, 160, 160, 0.8);
  }
</style>

<div
  bind:this={scrollerDiv}
  use:simplebarAction
  class="overflow-x-auto recently-played-scroller scroller"
  on:mousedown={handleMouseDown}
>
  <slot />
</div>