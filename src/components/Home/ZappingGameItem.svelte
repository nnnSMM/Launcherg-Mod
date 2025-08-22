<script lang="ts">
  import { link } from "svelte-spa-router";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import tippy, { type Instance, type Props as TippyOption } from "tippy.js";
  import { formatLastPlayed, formatPlayTime } from "@/lib/utils";

  export let collectionElement: CollectionElement;
  export let objectFit: "contain" | "cover" = "contain";
  export let containerClass = "";

  const imgSrc = convertFileSrc(collectionElement.thumbnail);

  // Prepare content for the tooltip
  const lastPlayed = formatLastPlayed(collectionElement.lastPlayAt);
  const playTime = formatPlayTime(collectionElement.totalPlayTimeSeconds);

  const tooltipContent = `
    <div class="p-2 space-y-1 text-left">
      <div class="text-sm text-text-secondary">${collectionElement.brandname}</div>
      <div class="text-lg text-text-primary font-bold">${collectionElement.gamename}</div>
      ${lastPlayed ? `<div class="text-sm text-text-secondary">最終プレイ: ${lastPlayed}</div>` : ""}
      ${playTime ? `<div class="text-sm text-text-secondary">プレイ時間: ${playTime}</div>` : ""}
    </div>
  `;

  // Tippy.js options
  const tooltipOptions: Partial<TippyOption> = {
    content: tooltipContent,
    allowHTML: true,
    delay: [1000, 0],
    placement: "right",
    offset: [-40, 15], // [skidding, distance] - negative skidding moves it "up"
    theme: "image-bg", // Use a single, combined theme
    arrow: false,
    onShow(instance: Instance) {
      // Set the CSS custom property on the tippy box to the current game's thumbnail
      const box = instance.popper.querySelector('.tippy-box');
      if (box instanceof HTMLElement) {
        box.style.setProperty('--tooltip-bg-image', `url("${imgSrc}")`);
      }
    },
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
          src={imgSrc}
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
