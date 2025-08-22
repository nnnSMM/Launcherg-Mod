<svelte:options accessors />

<script lang="ts">
  import SimpleBar from "simplebar";
  import { createEventDispatcher, onMount } from "svelte";
  import ButtonIcon from "./ButtonIcon.svelte";

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>();

  let scrollEl: HTMLElement | null = null;
  let simplebarInstance: SimpleBar | null = null;
  let showLeftButton = false;
  let showRightButton = false;

  const simplebarAction = (node: HTMLElement) => {
    simplebarInstance = new SimpleBar(node, {
      scrollbarMinSize: 64,
    });

    scrollEl = simplebarInstance.getScrollElement();

    const checkScrollButtons = () => {
      if (!scrollEl) return;
      const { scrollLeft, scrollWidth, clientWidth } = scrollEl;
      showLeftButton = scrollLeft > 0;
      showRightButton = scrollLeft < scrollWidth - clientWidth - 1; // 1px buffer for precision issues
    };

    scrollEl?.addEventListener("scroll", (e) => {
      dispatcher("scroll", { event: e });
      checkScrollButtons();
    });

    // Initial check
    onMount(() => {
      // Use timeout to ensure DOM is fully rendered
      setTimeout(checkScrollButtons, 100);
    });

    const observer = new ResizeObserver(checkScrollButtons);
    observer.observe(node);


    return {
      destroy: () => {
        scrollEl?.removeEventListener("scroll", dispatcher);
        observer.disconnect();
      },
    };
  };

  export let scrollBy = (options?: ScrollToOptions | undefined): void => {
    scrollEl?.scrollBy(options);
  };

  const handleScrollLeft = () => {
    const amount = scrollEl ? scrollEl.clientWidth * 0.8 : 300;
    scrollBy({ left: -amount, behavior: "smooth" });
  };

  const handleScrollRight = () => {
    const amount = scrollEl ? scrollEl.clientWidth * 0.8 : 300;
    scrollBy({ left: amount, behavior: "smooth" });
  };
</script>

<div class="w-full min-w-0 relative group">
  <div use:simplebarAction class="overflow-x-auto scroll-smooth">
    <slot />
  </div>

  <!-- Left Scroll Button -->
  {#if showLeftButton}
    <div
      class="absolute left-0 top-0 h-full flex items-center justify-center bg-gradient-to-r from-bg-primary to-transparent opacity-0 group-hover:opacity-100 transition-opacity"
    >
      <ButtonIcon
        icon="i-material-symbols-arrow-back-ios-new-rounded"
        on:click={handleScrollLeft}
        class="ml-2"
        ariaLabel="Scroll Left"
      />
    </div>
  {/if}

  <!-- Right Scroll Button -->
  {#if showRightButton}
    <div
      class="absolute right-0 top-0 h-full flex items-center justify-center bg-gradient-to-l from-bg-primary to-transparent opacity-0 group-hover:opacity-100 transition-opacity"
    >
      <ButtonIcon
        icon="i-material-symbols-arrow-forward-ios-rounded"
        on:click={handleScrollRight}
        class="mr-2"
        ariaLabel="Scroll Right"
      />
    </div>
  {/if}
</div>
