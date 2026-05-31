<script lang="ts">
  import type {
    CollectionElement,
    PlayStatus as PlayStatusType,
  } from "@/lib/types";
  import { PlayStatus as PSConst } from "@/lib/types";
  import { createEventDispatcher } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { theme } from "@/store/theme";
  import Icon from "/icon.png";

  export let game: CollectionElement;
  export let isSelected: boolean = false;
  export let previewTargetPlayStatus: PlayStatusType | undefined = undefined;

  export let itemHeight: number;
  export let targetImageWidth: number;
  export let imageDisplayHeight: number;
  export let tilePadding: number = 8;

  $: bgColorUnselected = $theme === "light" ? "#E5E7EB" : "#21262d";
  $: bgColorSelected = $theme === "light" ? "#E5E7EB" : "#2d333b";
  $: bgColorUnselectedHover = $theme === "light" ? "#CBD5E1" : "#30363d";
  $: bgColorSelectedHover = $theme === "light" ? "#CBD5E1" : "#373e47";
  let unplayedColor = "#6e7681";
  $: unplayedColor = $theme === "light" ? "#6B7280" : "#6e7681";

  const textColorNormalBase = "#374151";
  const textColorSelected = "#1f2937";

  const dispatch = createEventDispatcher<{ toggle: void }>();

  $: actualTileWidth = targetImageWidth + tilePadding * 2;

  const playStatusInfo: Record<
    PlayStatusType,
    {
      label: string;
      icon: string;
      baseColorCode: string;
      selectedIconColor: string; // 選択時のアイコン色
    }
  > = {
    [PSConst.Unplayed]: {
      label: "未プレイ",
      icon: "i-material-symbols-play-circle-outline-rounded",
      baseColorCode: unplayedColor,
      selectedIconColor: "#FFFFFF",
    }, // 選択時アイコンは白
    [PSConst.Playing]: {
      label: "プレイ中",
      icon: "i-material-symbols-pause-circle-outline-rounded",
      baseColorCode: "#4299E1",
      selectedIconColor: "#FFFFFF",
    }, // 選択時アイコンは白
    [PSConst.Cleared]: {
      label: "クリア済み",
      icon: "i-material-symbols-check-circle-outline-rounded",
      baseColorCode: "#48BB78",
      selectedIconColor: "#FFFFFF",
    }, // 選択時アイコンは白
    [PSConst.Interrupted]: {
      label: "中断",
      icon: "i-material-symbols-stop-circle-outline-rounded",
      baseColorCode: "#D97706",
      selectedIconColor: "#FFFFFF",
    },
    [PSConst.LegacyShelved]: {
      label: "中断",
      icon: "i-material-symbols-stop-circle-outline-rounded",
      baseColorCode: "#D97706",
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
    baseColorCode: unplayedColor,
    selectedIconColor: "#FFFFFF",
  };
  $: currentStatusColor =
    displayPlayStatus === PSConst.Unplayed
      ? unplayedColor
      : currentStatusInfo.baseColorCode;

  $: thumbnailSrc = game.thumbnail
    ? convertFileSrc(game.thumbnail)
    : Icon;

  let imageHasError = false;
  $: if (game.thumbnail) imageHasError = false;

  const handleImageError = () => {
    imageHasError = true;
  };

  let isHovered = false;

  $: currentBgColor = isSelected ? bgColorSelected : bgColorUnselected;
  $: currentHoverBgColor = isSelected
    ? bgColorSelectedHover
    : bgColorUnselectedHover;

  $: tileBorderStyle = (() => {
    const ringWidth = isSelected ? 4 : 2;
    const color = currentStatusColor;

    if (isSelected) {
      return `box-shadow: 0 0 0 ${ringWidth}px ${color} !important; border-color: transparent !important;`;
    } else if (isHovered) {
      // 非選択時ホバー: 枠線を維持しつつ、背景色に合わせた少し濃い色にするか、透明のままか
      // 今回はホバー時もプレイ状況の色を枠線として維持する
      return `border-color: ${color};`; // 背景色が変わるので枠線は元の色を維持
    } else {
      return `border-color: ${color};`; // 通常の非選択時
    }
  })();

  $: visibleTileRingStyle = `box-shadow: 0 0 0 ${isSelected ? 4 : 2}px ${currentStatusColor} !important; border-color: transparent !important;`;

  $: textFillColor = isSelected
    ? textColorSelected
    : currentStatusColor;
  $: textFillColorForLabel = isSelected
    ? textColorSelected
    : displayPlayStatus === PSConst.Unplayed
      ? textColorNormalBase
      : currentStatusColor;
</script>

<button
  class="p-0 rounded-lg border-2 transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-primary
         focus-visible:ring-accent-accent flex flex-col items-center
         hover:scale-102 hover:shadow-md"
  style="
    width: {actualTileWidth}px;
    height: {itemHeight}px;
    padding: {tilePadding}px;
    z-index: {isHovered ? 10 : 1};
    background-color: {isHovered ? currentHoverBgColor : currentBgColor};
    {visibleTileRingStyle}
  "
  on:click={() => dispatch("toggle")}
  on:mouseenter={() => (isHovered = true)}
  on:mouseleave={() => (isHovered = false)}
>
  <div
    class="image-container relative overflow-hidden rounded bg-bg-primary flex items-center justify-center"
    style="width: {targetImageWidth}px; height: {imageDisplayHeight}px; margin-bottom: {tilePadding /
      2}px;"
  >
    {#if imageHasError || !game.thumbnail}
      <div
        class="text-xs text-text-tertiary p-1 text-center break-all flex items-center justify-center w-full h-full bg-bg-secondary"
      >
        <img
          src="/images/dummy_thumbnail.svg"
          alt=""
          class="w-full h-full object-cover opacity-50"
        />
      </div>
    {:else}
      <img
        src={thumbnailSrc}
        alt={game.gamename}
        class="w-full h-full object-cover"
        on:error={handleImageError}
        loading="lazy"
      />
    {/if}
    <div
      class="absolute top-1 right-1 p-0.5 rounded-full"
      style="background-color: {isSelected
        ? currentStatusColor
        : 'rgba(0,0,0,0.3)'};"
    >
      <div
        class="{currentStatusInfo.icon} w-4 h-4"
        style="color: {isSelected
          ? currentStatusInfo.selectedIconColor
          : '#FFFFFF'};"
      />
    </div>
  </div>

  <div class="w-full text-center mt-auto px-1">
    <p
      class="text-xs font-medium truncate"
      style="color: {textFillColor}; max-width: {targetImageWidth}px;"
    >
      {game.gamename}
    </p>
    <p class="text-xs text-opacity-90" style="color: {textFillColorForLabel};">
      {currentStatusInfo.label}
    </p>
  </div>
</button>
