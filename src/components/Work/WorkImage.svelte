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

  $: srcWithVersion = `${convertFileSrc(element.thumbnail)}?v=${
    element.updatedAt
  }`;

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
          dispatcher("update");
        }
      },
    },
  ];
</script>

<div class="relative" on:contextmenu={handleContextMenu}>
  <img
    alt="{element.gamename}_thumbnail"
    src={srcWithVersion}
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
