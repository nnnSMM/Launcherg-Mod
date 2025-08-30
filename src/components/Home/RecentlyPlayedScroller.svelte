<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import SimpleBar from "simplebar";
  import { onMount, onDestroy } from "svelte";

  export const scrollBy = (options: ScrollToOptions) => {
    if (options.left) {
      // Add a 'push' to the velocity. The value is arbitrary and chosen for feel.
      scrollVelocity += options.left > 0 ? 35 : -35;
      startAnimation();
    }
  };

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>();

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

  const MAX_SPEED = 35;
  const onWheel = (e: WheelEvent) => {
    if (!isHovering || !scrollEl) return;

    if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      e.preventDefault();
      // Accumulate velocity
      scrollVelocity += e.deltaY * 0.25; // Multiplier to control sensitivity
      scrollVelocity = Math.max(-MAX_SPEED, Math.min(MAX_SPEED, scrollVelocity));
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

<!-- Add a style block to control SimpleBar's appearance -->
<style>
  :global(.recently-played-scroller .simplebar-track.simplebar-vertical) {
    display: none !important;
  }
  :global(.recently-played-scroller .simplebar-scrollbar::before) {
    background-color: rgba(160, 160, 160, 0.8);
  }
</style>

<div
  use:simplebarAction
  class="overflow-x-auto recently-played-scroller"
  on:mouseenter={() => (isHovering = true)}
  on:mouseleave={() => (isHovering = false)}
>
  <slot />
</div>
