<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { commandUpdateGameImage } from "@/lib/command";
  import ContextMenu from "@/components/UI/ContextMenu.svelte";
  import { writable } from "svelte/store";

  export let name: string;
  export let src: string;
  export let elementId: number; // ゲームIDを親から受け取る

  let menu = {
    isOpen: false,
    x: 0,
    y: 0,
  };

  // 画像のキャッシュを無効化するためのバージョン管理
  const imageVersion = writable(0);
  $: srcWithVersion = `${src}?v=${$imageVersion}`;

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
      label: "サムネイルを変更...",
      onSelect: async () => {
        const selected = await open({
          multiple: false,
          filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg"] }],
        });
        if (typeof selected?.path === "string") {
          await commandUpdateGameImage(elementId, "thumbnail", selected.path);
          imageVersion.update(n => n + 1); // 画像を強制的にリロード
        }
      },
    },
  ];
</script>

<div class="relative" on:contextmenu={handleContextMenu}>
  <img alt="{name}_icon" src={srcWithVersion} class="object-contain w-full" />
</div>

{#if menu.isOpen}
  <ContextMenu
    x={menu.x}
    y={menu.y}
    options={menuOptions}
    on:close={() => (menu.isOpen = false)}
  />
{/if}
