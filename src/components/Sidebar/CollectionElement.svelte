<script lang="ts">
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { link, location } from "svelte-spa-router";
  import ContextMenu from "@/components/UI/ContextMenu.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { commandUpdateGameImage } from "@/lib/command";
  import { createEventDispatcher } from "svelte";
  import { formatLastPlayed } from "@/lib/utils";

  export let collectionElement: CollectionElement;

  const dispatcher = createEventDispatcher();

  let menu = {
    isOpen: false,
    x: 0,
    y: 0,
  };

  const withCacheBuster = (src: string, updatedAt: string) =>
    /^(blob:|data:)/.test(src) ? src : `${src}?v=${updatedAt}`;

  $: iconSrc = withCacheBuster(
    convertFileSrc(collectionElement.icon),
    collectionElement.updatedAt,
  );

  $: isActive = $location.includes(`/works/${collectionElement.id}`);
  $: lastPlayedText = formatLastPlayed(collectionElement.lastPlayAt);
  $: isLightTheme =
    document?.documentElement?.dataset.theme === "light";
  $: itemStateClass = isActive
    ? isLightTheme
      ? "bg-bg-button-hover/75"
      : "bg-bg-button-hover/55"
    : isLightTheme
      ? "hover:bg-bg-button-hover/50"
      : "hover:bg-bg-button-hover/35";

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    menu = {
      isOpen: true,
      x: e.clientX,
      y: e.clientY,
    };
  };

  const menuOptions = [
    {
      label: "アイコンを変更",
      onSelect: async () => {
        const selected = await open({
          multiple: false,
          filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg"] }],
        });
        if (typeof selected?.path === "string") {
          await commandUpdateGameImage(
            collectionElement.id,
            "icon",
            selected.path,
          );
          // アイコン変更後、画像の再フェッチを促すためupdatedAtを更新
          collectionElement.updatedAt = new Date().toISOString();
        }
      },
    },
  ];
</script>

<div
  class="flex items-center py-1 px-2 mx-2 rounded-lg transition-all overflow-hidden group relative {itemStateClass}"
  on:contextmenu={handleContextMenu}
>
  {#if isActive}
    <div
      class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-5 bg-accent-accent rounded-r-full"
    />
  {/if}
  <a
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    class="flex flex-1 w-full items-center gap-3 min-w-0 focus-visible:ring-2 focus-visible:ring-accent-accent rounded"
    aria-current={isActive ? "page" : undefined}
    title={collectionElement.gamename}
    use:link
  >
    <img
      alt="{collectionElement.gamename}_icon"
      src={iconSrc}
      class="h-5 w-5 rounded object-cover shadow-sm transition-transform group-hover:scale-105"
      loading="lazy"
      on:error={(e) => {
        const img = e.currentTarget;
        if (img instanceof HTMLImageElement) {
          img.src = "/images/dummy_thumbnail.svg";
        }
      }}
    />
    <div class="min-w-0 flex-1">
      <div
        class="text-xs font-medium text-text-secondary group-hover:text-text-primary truncate transition-colors"
      >
        {collectionElement.gamename}
      </div>
    </div>
  </a>
</div>

{#if menu.isOpen}
  <ContextMenu
    x={menu.x}
    y={menu.y}
    options={menuOptions}
    disableHover
    on:close={() => (menu.isOpen = false)}
  />
{/if}
