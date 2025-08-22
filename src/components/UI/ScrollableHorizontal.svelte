<svelte:options accessors />

<script lang="ts">
  import SimpleBar from "simplebar";
  import { onMount } from "svelte";
  import ButtonIcon from "./ButtonIcon.svelte";

  let scrollEl: HTMLElement | null = null;
  let contentEl: HTMLElement | null = null;

  let showLeftButton = false;
  let showRightButton = false;

  const simplebarAction = (node: HTMLElement) => {
    const simplebarInstance = new SimpleBar(node, {
      // Disable the default vertical scrollbar to prevent any interference
      autoHide: true,
    });
    scrollEl = simplebarInstance.getScrollElement();

    const checkScrollButtons = () => {
      if (!scrollEl || !contentEl) return;
      const buffer = 2; // Buffer for sub-pixel precision issues
      const { scrollLeft, clientWidth } = scrollEl;
      // Use the content element's width to get the true scrollable width
      const scrollWidth = contentEl.scrollWidth;
      showLeftButton = scrollLeft > buffer;
      showRightButton = scrollLeft < scrollWidth - clientWidth - buffer;
    };

    // Listen to scroll events on the SimpleBar element
    scrollEl?.addEventListener("scroll", checkScrollButtons);

    // Use ResizeObserver to detect size changes of the container and content
    const observer = new ResizeObserver(checkScrollButtons);
    observer.observe(node);
    if(contentEl) observer.observe(contentEl);

    // Initial check after mount to ensure layout is stable
    onMount(() => {
      setTimeout(checkScrollButtons, 200);
    });

    return {
      destroy: () => {
        if(scrollEl) scrollEl.removeEventListener("scroll", checkScrollButtons);
        observer.disconnect();
      },
    };
  };

  const handleScrollLeft = () => {
    if (!scrollEl) return;
    const amount = scrollEl.clientWidth * 0.8;
    scrollEl.scrollBy({ left: -amount, behavior: "smooth" });
  };

  const handleScrollRight = () => {
    if (!scrollEl) return;
    const amount = scrollEl.clientWidth * 0.8;
    scrollEl.scrollBy({ left: amount, behavior: "smooth" });
  };
</script>

<div class="relative">
  <!-- This div contains the buttons, positioned relative to the container -->
  <div class="absolute right-0 -top-8 flex items-center space-x-1 z-10">
      <ButtonIcon
        icon="i-material-symbols-arrow-back-ios-new-rounded"
        on:click={handleScrollLeft}
        ariaLabel="Scroll Left"
        disabled={!showLeftButton}
        class="disabled:opacity-25 disabled:cursor-not-allowed"
      />
      <ButtonIcon
        icon="i-material-symbols-arrow-forward-ios-rounded"
        on:click={handleScrollRight}
        ariaLabel="Scroll Right"
        disabled={!showRightButton}
        class="disabled:opacity-25 disabled:cursor-not-allowed"
      />
  </div>

  <!-- The scrollable area -->
  <div use:simplebarAction class="overflow-x-auto scroll-smooth">
    <div bind:this={contentEl}>
      <slot />
    </div>
  </div>
</div>
