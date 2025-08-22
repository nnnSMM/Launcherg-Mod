<script lang="ts">
  import { link } from "svelte-spa-router";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import tippy, { type Props as TippyOption } from "tippy.js";
  import { formatLastPlayed, formatPlayTime } from "@/lib/utils";

  export let collectionElement: CollectionElement;
  export let objectFit: "contain" | "cover" = "contain";
  export let containerClass = "";

  // Prepare content for the tooltip
  const lastPlayed = formatLastPlayed(collectionElement.lastPlayAt);
  const playTime = formatPlayTime(collectionElement.totalPlayTimeSeconds);

  // Note: UnoCSS classes used here for styling must be available in the global scope
  // as tippy.js renders the tooltip outside the component.
  const tooltipContent = `
    <div class="p-2 space-y-1 text-left">
      <div class="text-xs text-text-tertiary">${collectionElement.brandname}</div>
      <div class="text-base text-text-primary font-bold">${collectionElement.gamename}</div>
      ${lastPlayed ? `<div class="text-xs text-text-tertiary">最終プレイ: ${lastPlayed}</div>` : ""}
      ${playTime ? `<div class="text-xs text-text-tertiary">プレイ時間: ${playTime}</div>` : ""}
    </div>
  `;

  // Tippy.js options
  const tooltipOptions: Partial<TippyOption> = {
    content: tooltipContent,
    allowHTML: true,
    delay: [1000, 0], // 1000ms to show, 0ms to hide
    placement: "auto",
    theme: "default", // Assuming a default theme is defined somewhere
    arrow: true,
  };

  const tooltipAction = (node: HTMLElement) => {
    const tp = tippy(node, tooltipOptions);
    return {
      destroy() {
        tp.destroy();
      },
    };
  };
</script>

<div class={`w-full h-full ${containerClass}`}>
  <slot name="info" />
  <div
    use:tooltipAction
    class="hover:scale-[1.025] hover:shadow-md focus-within:scale-105 focus-within:shadow-md transition-all cursor-pointer w-full h-full relative hover:z-10"
  >
    <a
      tabIndex={0}
      href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
      use:link
      class="w-full h-full block"
    >
      {#if collectionElement.thumbnail}
        <img
          decoding="async"
          class="rounded w-full h-full"
          class:object-contain={objectFit === "contain"}
          class:object-cover={objectFit === "cover"}
          src={convertFileSrc(collectionElement.thumbnail)}
          alt={`${collectionElement.gamename}のサムネイル`}
        />
      {:else}
        <div
          class="text-(body text-primary) font-bold px-6 rounded border bg-bg-primary w-full h-full flex items-center justify-center"
        >
          {collectionElement.gamename}
        </div>
      {/if}
    </a>
  </div>
</div>
