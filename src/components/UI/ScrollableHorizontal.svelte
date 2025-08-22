<script lang="ts">
  import SimpleBar from "simplebar";
  import { onMount } from "svelte";

  let scrollEl: HTMLElement | null = null;
  let isHovering = false;

  const simplebarAction = (node: HTMLElement) => {
    // Initialize SimpleBar.
    // The 'y' axis is disabled by setting a class that hides its scrollbar.
    const simplebarInstance = new SimpleBar(node, {
      classNames: {
        scrollbar: 'simplebar-scrollbar-custom',
        track: 'simplebar-track-custom'
      }
    });
    scrollEl = simplebarInstance.getScrollElement();
  };

  const onWheel = (e: WheelEvent) => {
    // Only act if the mouse is over the component
    if (!isHovering || !scrollEl) return;

    // If there is significant vertical scroll, use it to scroll horizontally
    if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      // Prevent the main page from scrolling vertically
      e.preventDefault();
      // Scroll the element horizontally, with a multiplier for speed
      scrollEl.scrollBy({ left: e.deltaY * 2, behavior: 'auto' });
    }
  };

  onMount(() => {
    // Add the wheel listener to the window to capture the event
    // passive: false is required to be able to call e.preventDefault()
    window.addEventListener("wheel", onWheel, { passive: false });

    return () => {
      window.removeEventListener("wheel", onWheel);
    };
  });
</script>

<!-- Add a style block to control SimpleBar's appearance -->
<style>
  :global(.simplebar-track-custom.simplebar-vertical) {
    display: none !important;
  }
  /* You can add styles here to make the horizontal scrollbar more prominent if needed */
  :global(.simplebar-scrollbar-custom::before) {
    background-color: #a0a0a0; /* Example color */
  }
</style>

<div
  use:simplebarAction
  class="overflow-x-auto"
  on:mouseenter={() => (isHovering = true)}
  on:mouseleave={() => (isHovering = false)}
>
  <slot />
</div>
