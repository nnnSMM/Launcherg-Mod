<script lang="ts">
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { link, location } from "svelte-spa-router";
  import ContextMenu from "@/components/UI/ContextMenu.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { commandUpdateGameImage } from "@/lib/command";
  import { createEventDispatcher } from "svelte";

  export let collectionElement: CollectionElement;

  const dispatcher = createEventDispatcher();

  let menu = {
    isOpen: false,
    x: 0,
    y: 0,
  };

  $: iconSrc = `${convertFileSrc(collectionElement.icon)}?v=${
    collectionElement.updatedAt
  }`;

  $: isActive = $location.includes(`/works/${collectionElement.id}`);

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
          window.location.reload();
        }
      },
    },
  ];
</script>

<div
  class="flex items-center py-1.5 px-2 mx-2 rounded-lg transition-all hover:bg-white/5 overflow-hidden group relative"
  on:contextmenu={handleContextMenu}
  class:bg-white-10={isActive}
>
  {#if isActive}
    <div
      class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-accent-accent rounded-r-full"
    />
  {/if}
  <a
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    class="flex-(~ 1) w-full items-center gap-3"
    use:link
  >
    <img
      alt="{collectionElement.gamename}_icon"
      src={iconSrc}
      class="h-8 w-8 rounded-md shadow-sm transition-transform group-hover:scale-105"
      loading="lazy"
    />
    <div
      class="text-sm font-medium text-text-secondary group-hover:text-text-primary truncate transition-colors"
    >
      {collectionElement.gamename}
    </div>
  </a>
</div>

{#if menu.isOpen}
  <ContextMenu
    x={menu.x}
    y={menu.y}
    options={menuOptions}
    on:close={() => (menu.isOpen = false)}
  />
{/if}
