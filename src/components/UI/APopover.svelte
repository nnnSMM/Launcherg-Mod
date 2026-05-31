<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { fly } from "svelte/transition";

  export let isRelativeRoot = true;
  export let panelClass = "";
  export let placement: "auto" | "top" | "bottom" = "auto";

  let open = false;
  let root: HTMLDivElement | null = null;
  let buttonAnchor: HTMLDivElement | null = null;
  let panelElement: HTMLDivElement | null = null;
  let panelStyle = "";
  let pointerDownStartedInside = false;

  const portal = (node: HTMLElement) => {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      },
    };
  };

  const close = (_?: unknown) => {
    open = false;
  };

  const updatePanelPosition = async () => {
    await tick();
    if (!buttonAnchor) return;

    const rect = buttonAnchor.getBoundingClientRect();
    const gap = 8;
    const margin = 8;
    let top = rect.bottom + gap;
    let left = rect.left;

    if (panelClass.includes("right-0")) {
      const panelWidth = panelElement?.offsetWidth ?? 0;
      left = rect.right - panelWidth;
    }

    const panelHeight = panelElement?.offsetHeight ?? 0;
    const panelWidth = panelElement?.offsetWidth ?? 0;

    if (placement === "top" && panelHeight > 0) {
      top = Math.max(margin, rect.top - panelHeight - gap);
    } else if (
      placement === "auto" &&
      panelHeight > 0 &&
      top + panelHeight > window.innerHeight - margin
    ) {
      top = Math.max(margin, rect.top - panelHeight - gap);
    }

    if (panelWidth > 0) {
      left = Math.min(left, window.innerWidth - panelWidth - margin);
    }
    left = Math.max(margin, left);

    panelStyle = `top: ${top}px; left: ${left}px; right: auto; bottom: auto;`;
  };

  const toggle = async () => {
    open = !open;
    if (open) {
      await updatePanelPosition();
    }
  };

  const handleWindowPointerDown = (event: PointerEvent) => {
    const target = event.target as Node;
    pointerDownStartedInside =
      root?.contains(target) === true || panelElement?.contains(target) === true;
  };

  const handleWindowClick = (event: MouseEvent) => {
    const target = event.target as Node;
    if (root?.contains(target) || panelElement?.contains(target)) {
      pointerDownStartedInside = false;
      return;
    }
    if (pointerDownStartedInside) {
      pointerDownStartedInside = false;
      return;
    }
    close();
  };

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      close();
    }
  };

  onMount(() => {
    window.addEventListener("pointerdown", handleWindowPointerDown, { capture: true });
    window.addEventListener("click", handleWindowClick, { capture: true });
    window.addEventListener("resize", updatePanelPosition);
    window.addEventListener("scroll", updatePanelPosition, true);
    window.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("pointerdown", handleWindowPointerDown, { capture: true });
    window.removeEventListener("click", handleWindowClick, { capture: true });
    window.removeEventListener("resize", updatePanelPosition);
    window.removeEventListener("scroll", updatePanelPosition, true);
    window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div
  bind:this={root}
  class={isRelativeRoot ? "relative" : ""}
  on:click|stopPropagation
  on:keydown|stopPropagation
>
  <div
    bind:this={buttonAnchor}
    class="inline-block"
    role="presentation"
    on:click={toggle}
    on:keydown={(event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        toggle();
      }
    }}
  >
    <slot name="button" {open} {close} />
  </div>
  {#if open}
    <div
      bind:this={panelElement}
      use:portal
      transition:fly={{ y: -40, duration: 150 }}
      class="glass-menu-surface fixed z-10000 w-max max-w-[calc(100vw-16px)] overflow-hidden rounded-md {panelClass}"
      style={panelStyle}
      on:click|stopPropagation
      on:keydown|stopPropagation
      on:contextmenu|stopPropagation
    >
      <slot {open} {close} />
    </div>
  {/if}
</div>
