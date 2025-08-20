<script lang="ts">
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { link } from "svelte-spa-router";
  import ContextMenu from "@/components/UI/ContextMenu.svelte";
  import { writable } from "svelte/store";
  import { open } from "@tauri-apps/plugin-dialog";
  import { commandUpdateGameImage } from "@/lib/command";

  export let collectionElement: CollectionElement;

  // --- Start of changes ---

  let menu = {
    isOpen: false,
    x: 0,
    y: 0,
  };

  const imageVersion = writable(0);
  // Re-compute the src with version whenever the original src or the version changes
  $: iconSrc = `${convertFileSrc(collectionElement.icon)}?v=${$imageVersion}`;

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
      label: "アイコンを変更...",
      onSelect: async () => {
        const selected = await open({
          multiple: false,
          filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg"] }],
        });
        if (typeof selected?.path === "string") {
          await commandUpdateGameImage(collectionElement.id, "icon", selected.path);
          imageVersion.update((n) => n + 1); // Force reload
        }
      },
    },
  ];
  // --- End of changes ---
</script>

<div
  class="flex items-center py-1 pl-2 rounded transition-all hover:bg-bg-secondary overflow-hidden"
  on:contextmenu={handleContextMenu}
>
  <a
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    class="flex-(~ 1) h-12 w-full items-center gap-2 pr-2"
    use:link
  >
    <img
      alt="{collectionElement.gamename}_icon"
      src={iconSrc}
      class="h-10 w-10 rounded"
      loading="lazy"
    />
    <div class="text-(body text-primary) font-bold max-h-full">
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
