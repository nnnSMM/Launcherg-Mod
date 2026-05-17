import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import sveltePreprocess from "svelte-preprocess";
import UnoCSS from "unocss/vite";
import { fileURLToPath } from "node:url";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  base: "./",
  plugins: [
    UnoCSS(),
    svelte({
      preprocess: [
        sveltePreprocess({
          typescript: true,
        }),
      ],
    }),
  ],

  resolve: {
    preserveSymlinks: true,
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "@tauri-apps/api/core": fileURLToPath(new URL("./src/mock/tauri-core.ts", import.meta.url)),
      "@tauri-apps/api/window": fileURLToPath(new URL("./src/mock/tauri-window.ts", import.meta.url)),
      "@tauri-apps/api/event": fileURLToPath(new URL("./src/mock/tauri-event.ts", import.meta.url)),
      "@tauri-apps/plugin-dialog": fileURLToPath(new URL("./src/mock/tauri-dialog.ts", import.meta.url)),
      "@tauri-apps/plugin-shell": fileURLToPath(new URL("./src/mock/tauri-shell.ts", import.meta.url)),
      "@tauri-apps/plugin-fs": fileURLToPath(new URL("./src/mock/tauri-fs.ts", import.meta.url)),
      "@tauri-apps/plugin-clipboard-manager": fileURLToPath(new URL("./src/mock/tauri-clipboard.ts", import.meta.url)),
      "@tauri-apps/plugin-http": fileURLToPath(new URL("./src/mock/tauri-http.ts", import.meta.url)),
      "@/store/works": fileURLToPath(new URL("./src/mock/works.ts", import.meta.url)),
    },
  },

  build: {
    outDir: "docs/demo",
    emptyOutDir: true,
  },
}));
