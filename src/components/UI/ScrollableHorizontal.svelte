<svelte:options accessors />

<script lang="ts">
  import SimpleBar from "simplebar";
  import { createEventDispatcher } from "svelte";

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>();

  let isHover = false;

  const simplebar = (node: HTMLElement) => {
    const instance = new SimpleBar(node, {
      scrollbarMinSize: 64,
    });

    const onScroll = (e: Event) => {
      dispatcher("scroll", { event: e });
    };
    instance.getScrollElement()?.addEventListener("scroll", onScroll);

    const onWheel = (e: WheelEvent) => {
      if (isHover) {
        e.preventDefault();
        instance
          .getScrollElement()
          ?.scrollBy({ left: e.deltaY * 5, behavior: "smooth" });
      }
    };
    window.addEventListener("wheel", onWheel, { passive: false });

    const element = instance.getScrollElement();
    if (element) {
      scrollBy = (options?: ScrollToOptions | undefined) => {
        element.scrollBy(options);
      };
    }
    return {
      destroy: () => {
        window.removeEventListener("wheel", onWheel);
        instance.getScrollElement()?.removeEventListener("scroll", onScroll);
        instance.unMount();
        scrollBy = () => undefined;
      },
    };
  };

  export let scrollBy = (options?: ScrollToOptions | undefined): void => {
    console.warn("scrollBy is not initialized");
  };
</script>

<div class="w-full min-w-0">
  <div
    use:simplebar
    class="overflow-x-auto scroll-smooth"
    on:mouseenter={() => (isHover = true)}
    on:mouseleave={() => (isHover = false)}
  >
    <slot />
  </div>
</div>
