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

  export let itemHeight: number;
  export let targetImageWidth: number;
  export let imageDisplayHeight: number;
  export let tilePadding: number = 8;

  /* アプリのダークサーフェスに合わせる（従来のライトグレーは浮いて見えるため） */
  const bgColorUnselected = "#21262d";
  const bgColorSelected = "#2d333b";
  const bgColorUnselectedHover = "#30363d";
  const bgColorSelectedHover = "#373e47";

  /* 背景 #21262d 上で判読性と抑えたトーンのバランス */
  const textColorTitleNormal = "#8b949e";
  const textColorTitleSelected = "#e6edf3";
  const textColorMetaNormal = "#6e7681";
  const textColorMetaSelected = "#adbac7";

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
      baseColorCode: "#A0AEC0",
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
  };

  $: displayPlayStatus =
    isSelected && previewTargetPlayStatus !== undefined
      ? previewTargetPlayStatus
      : game.playStatus;
  $: currentStatusInfo = playStatusInfo[displayPlayStatus] || {
    label: "不明",
    icon: "i-material-symbols-help-outline-rounded",
    baseColorCode: "#A0AEC0",
    selectedIconColor: "#FFFFFF",
  };

  $: thumbnailSrc = game.thumbnail
    ? convertFileSrc(game.thumbnail)
    : "/icon.png";

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
    const color = currentStatusInfo.baseColorCode;

    if (isSelected) {
      return `box-shadow: 0 0 0 ${ringWidth}px ${color}; border-color: transparent;`;
    } else if (isHovered) {
      // 非選択時ホバー: 枠線を維持しつつ、背景色に合わせた少し濃い色にするか、透明のままか
      // 今回はホバー時もプレイ状況の色を枠線として維持する
      return `border-color: ${color};`; // 背景色が変わるので枠線は元の色を維持
    } else {
      return `border-color: ${color};`; // 通常の非選択時
    }
  })();

  /* 状態色は枠・バッジに任せ、タイトルは常に抑えたトーン */
  $: textFillColor = isSelected ? textColorTitleSelected : textColorTitleNormal;
  $: textFillColorForLabel = isSelected
    ? textColorMetaSelected
    : textColorMetaNormal;
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
    {tileBorderStyle}
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
        class="text-(xs text-tertiary) p-1 text-center break-all flex items-center justify-center w-full h-full bg-bg-secondary"
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
        ? currentStatusInfo.baseColorCode
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
      class="text-(xs) font-medium truncate"
      style="color: {textFillColor}; max-width: {targetImageWidth}px;"
    >
      {game.gamename}
    </p>
    <p class="text-(xs opacity-90)" style="color: {textFillColorForLabel};">
      {currentStatusInfo.label}
    </p>
  </div>
</button>
