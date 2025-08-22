<svelte:options accessors />

<script lang="ts">
  import SimpleBar from "simplebar";
  import { onMount } from "svelte";
  import ButtonIcon from "./ButtonIcon.svelte";

  let scrollEl: HTMLElement | null = null;
  let contentEl: HTMLElement | null = null;
  let showLeftButton = false;
  let showRightButton = true; // Assume we can scroll right initially

  const simplebarAction = (node: HTMLElement) => {
    const simplebarInstance = new SimpleBar(node, {
      scrollbarMinSize: 64,
    });
    scrollEl = simplebarInstance.getScrollElement();

    const checkScrollButtons = () => {
      if (!scrollEl || !contentEl) return;
      const buffer = 2;
      const { scrollLeft, clientWidth } = scrollEl;
      const scrollWidth = contentEl.scrollWidth; // Use the content's scroll width
      showLeftButton = scrollLeft > buffer;
      showRightButton = scrollLeft < scrollWidth - clientWidth - buffer;
    };

    scrollEl?.addEventListener("scroll", checkScrollButtons);

    const observer = new ResizeObserver(checkScrollButtons);
    observer.observe(node);
    if(contentEl) observer.observe(contentEl);

    onMount(() => {
      setTimeout(checkScrollButtons, 150);
    });

    return {
      destroy: () => {
        scrollEl?.removeEventListener("scroll", checkScrollButtons);
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

<div class="w-full min-w-0 relative group">
  <div use:simplebarAction class="overflow-x-auto scroll-smooth">
    <div bind:this={contentEl}>
      <slot />
    </div>
  </div>

  <!-- Scroll Buttons Container -->
  <div
    class="absolute right-2 -top-9 h-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity z-10 pointer-events-none"
  >
    <div class="flex items-center space-x-1 bg-bg-primary rounded-full p-1 shadow pointer-events-auto">
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
  </div>
</div>
