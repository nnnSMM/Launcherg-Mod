<script lang="ts">
  import SimpleBar from "simplebar";
  import { onMount, onDestroy } from "svelte";

  let scrollEl: HTMLElement | null = null;
  let isHovering = false;

  // Inertial scroll state
  let scrollVelocity = 0;
  let animationFrameId: number | null = null;
  const DAMPING_FACTOR = 0.92; // Friction
  const MIN_VELOCITY = 0.1; // Stop when slow enough

  const simplebarAction = (node: HTMLElement) => {
    const simplebarInstance = new SimpleBar(node, {});
    scrollEl = simplebarInstance.getScrollElement();
  };

  const startAnimation = () => {
    if (animationFrameId) return; // Already running
    tick();
  };

  const stopAnimation = () => {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }
  };

  const tick = () => {
    if (!scrollEl) return;

    // Apply velocity
    scrollEl.scrollLeft += scrollVelocity;

    // Apply damping
    scrollVelocity *= DAMPING_FACTOR;

    // Stop animation if velocity is low
    if (Math.abs(scrollVelocity) > MIN_VELOCITY) {
      animationFrameId = requestAnimationFrame(tick);
    } else {
      stopAnimation();
    }
  };

  const onWheel = (e: WheelEvent) => {
    if (!isHovering || !scrollEl) return;

    if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      e.preventDefault();
      // Accumulate velocity
      scrollVelocity += e.deltaY * 0.25; // Multiplier to control sensitivity
      startAnimation();
    }
  };

  onMount(() => {
    window.addEventListener("wheel", onWheel, { passive: false });
  });

  onDestroy(() => {
    window.removeEventListener("wheel", onWheel);
    stopAnimation();
  });
</script>

<!-- Add a unique class to the root to scope the global styles -->
<div class="horizontal-scroll-container">
  <div
    use:simplebarAction
    class="overflow-x-auto"
    on:mouseenter={() => (isHovering = true)}
    on:mouseleave={() => (isHovering = false)}
  >
    <slot />
  </div>
</div>

<!-- Scope the global styles to this component's unique class -->
<style>
  .horizontal-scroll-container :global(.simplebar-track.simplebar-vertical) {
    display: none !important;
  }
  .horizontal-scroll-container :global(.simplebar-scrollbar::before) {
    background-color: rgba(160, 160, 160, 0.8);
  }
</style>
