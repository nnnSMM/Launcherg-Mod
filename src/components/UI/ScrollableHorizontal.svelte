<script lang="ts">
  import SimpleBar from "simplebar";
  import { onMount } from "svelte";

  let scrollEl: HTMLElement | null = null;
  let isHovering = false;

  const simplebarAction = (node: HTMLElement) => {
    const simplebarInstance = new SimpleBar(node, {
      autoHide: true,
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
      // Scroll the element horizontally
      scrollEl.scrollBy({ left: e.deltaY, behavior: 'auto' });
    }
  };

  onMount(() => {
    // Add the wheel listener to the window to capture the event
    window.addEventListener("wheel", onWheel, { passive: false });

    return () => {
      window.removeEventListener("wheel", onWheel);
    };
  });
</script>

<div
  use:simplebarAction
  class="overflow-x-auto"
  on:mouseenter={() => (isHovering = true)}
  on:mouseleave={() => (isHovering = false)}
>
  <slot />
</div>
