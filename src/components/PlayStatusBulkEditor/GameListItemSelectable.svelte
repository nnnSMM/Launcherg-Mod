<script lang="ts">
  import type {
    CollectionElement,
    PlayStatus as PlayStatusType,
  } from "@/lib/types";
  import { PlayStatus as PSConst } from "@/lib/types";
  import { createEventDispatcher } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";

  export let game: CollectionElement;
  export let isSelected: boolean = false;
  export let previewTargetPlayStatus: PlayStatusType | undefined = undefined;

  const dispatch = createEventDispatcher<{ toggle: void }>();

  const playStatusInfo: Record<
    PlayStatusType,
    {
      label: string;
      icon: string;
      textColorClass: string;
      bgColorClass?: string;
      borderColorClass?: string;
      baseColorCode: string;
      selectedIconColor: string;
    }
  > = {
    [PSConst.Unplayed]: {
      label: "未プレイ",
      icon: "i-material-symbols-play-circle-outline-rounded",
      textColorClass: "text-gray-500 dark:text-gray-400",
      bgColorClass: "bg-gray-100 dark:bg-gray-700",
      borderColorClass: "border-gray-400",
      baseColorCode: "#A0AEC0",
      selectedIconColor: "#FFFFFF",
    },
    [PSConst.Playing]: {
      label: "プレイ中",
      icon: "i-material-symbols-pause-circle-outline-rounded",
      textColorClass: "text-blue-600 dark:text-blue-400",
      bgColorClass: "bg-blue-100 dark:bg-blue-800",
      borderColorClass: "border-blue-500",
      baseColorCode: "#4299E1",
      selectedIconColor: "#FFFFFF",
    },
    [PSConst.Cleared]: {
      label: "クリア済み",
      icon: "i-material-symbols-check-circle-outline-rounded",
      textColorClass: "text-green-600 dark:text-green-400",
      bgColorClass: "bg-green-100 dark:bg-green-800",
      borderColorClass: "border-green-500",
      baseColorCode: "#48BB78",
      selectedIconColor: "#FFFFFF",
    },
  };

  $: displayPlayStatus =
    isSelected && previewTargetPlayStatus !== undefined
      ? previewTargetPlayStatus
      : game.playStatus;
  $: currentStatusInfo = playStatusInfo[displayPlayStatus] || {
    label: "不明",
    icon: "i-material-symbols-help-outline-rounded",
    textColorClass: "text-gray-500 dark:text-gray-400",
    bgColorClass: "bg-gray-100 dark:bg-gray-700",
    borderColorClass: "border-gray-400",
    baseColorCode: "#A0AEC0",
    selectedIconColor: "#FFFFFF",
  };

  $: imageSrcToDisplay = game.icon ? convertFileSrc(game.icon) : "/icon.png";
  let imageHasError = false;
  $: if (game.icon) imageHasError = false;
  const handleImageError = () => {
    imageHasError = true;
  };

  let isHovered = false;

  const bgColorUnselected = "#21262d";
  const bgColorSelected = "#2d333b";
  const bgColorHover = "#30363d";

  $: currentBgColor = isSelected ? bgColorSelected : bgColorUnselected;
  $: currentHoverBgColor = isHovered ? bgColorHover : currentBgColor;

  $: tileBorderStyle = (() => {
    const color = currentStatusInfo.baseColorCode;
    if (isSelected) {
      return `border-color: ${color}; box-shadow: 0 0 0 2px ${color};`;
    } else if (isHovered) {
      return `border-color: ${color};`;
    } else {
      return `border-color: ${color};`;
    }
  })();

  const textColorTitleNormal = "#8b949e";
  const textColorTitleSelected = "#e6edf3";
  const textColorMetaNormal = "#6e7681";
  const textColorMetaSelected = "#adbac7";

  $: textFillColor = isSelected ? textColorTitleSelected : textColorTitleNormal;
  $: textFillColorForLabel = isSelected
    ? textColorMetaSelected
    : textColorMetaNormal;
</script>

<button
  class="w-full flex items-center p-3 rounded-lg border transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-1 focus-visible:ring-offset-bg-primary focus-visible:ring-accent-accent"
  style="background-color: {currentHoverBgColor};
         {tileBorderStyle}
         min-height: 4rem;
         z-index: {isHovered ? 10 : 1};"
  class:shadow-md={isHovered || isSelected}
  on:click={() => dispatch("toggle")}
  on:mouseenter={() => (isHovered = true)}
  on:mouseleave={() => (isHovered = false)}
  title={`${game.gamename}\n現在の状態: ${currentStatusInfo.label}`}
>
  <div
    class="flex-shrink-0 w-10 h-10 mr-3 rounded-md overflow-hidden bg-bg-secondary flex items-center justify-center border border-border-primary"
  >
    {#if imageHasError || !game.icon}
      <div class="i-material-symbols-image-not-supported-outline text-3xl text-ui-tertiary"></div>
    {:else}
      <img
        src={imageSrcToDisplay}
        alt="{game.gamename} icon"
        class="w-full h-full object-contain"
        on:error={handleImageError}
        loading="lazy"
      />
    {/if}
  </div>

  <div class="flex-1 min-w-0">
    <p class="text-sm font-medium truncate" style="color: {textFillColor};">
      {game.gamename}
    </p>
    <p class="text-xs leading-snug" style="color: {textFillColorForLabel};">
      {currentStatusInfo.label}
    </p>
  </div>

  <div class="ml-auto pl-2 flex-shrink-0">
    {#if isSelected}
      <div
        class="i-material-symbols-check-circle-rounded w-5 h-5"
        style="color: {currentStatusInfo.baseColorCode};"
      ></div>
    {:else}
      <div class="i-material-symbols-circle-outline w-5 h-5 text-ui-tertiary opacity-80"></div>
    {/if}
  </div>
</button>
