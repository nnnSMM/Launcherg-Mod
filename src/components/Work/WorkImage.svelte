<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { commandUpdateGameImage } from "@/lib/command";
  import ContextMenu from "@/components/UI/ContextMenu.svelte";
  import type { CollectionElement } from "@/lib/types";
  import { createEventDispatcher } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";

  export let element: CollectionElement;
  const dispatcher = createEventDispatcher();

  let menu = {
    isOpen: false,
    x: 0,
    y: 0,
  };

  $: srcWithVersion =
    element.thumbnail && element.thumbnail.trim() !== ""
      ? `${convertFileSrc(element.thumbnail)}?v=${element.updatedAt}`
      : "/images/dummy_thumbnail.svg";

  const handleError = (e: Event) => {
    const img = e.target as HTMLImageElement;
    img.src = "/images/dummy_thumbnail.svg";
  };

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
      label: "サムネイルを変更",
      onSelect: async () => {
        const selected = await open({
          multiple: false,
          filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg"] }],
        });
        if (typeof selected?.path === "string") {
          await commandUpdateGameImage(element.id, "thumbnail", selected.path);
          window.location.reload();
        }
      },
    },
  ];
</script>

<div class="relative" on:contextmenu={handleContextMenu}>
  <img
    alt="{element.gamename}_thumbnail"
    src={srcWithVersion}
    on:error={handleError}
    class="object-contain w-full"
  />
</div>

{#if menu.isOpen}
  <ContextMenu
    x={menu.x}
    y={menu.y}
    options={menuOptions}
    on:close={() => (menu.isOpen = false)}
  />
{/if}
