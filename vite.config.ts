import { defineConfig, type Plugin, type ResolvedConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import sveltePreprocess from "svelte-preprocess";
import UnoCSS from "unocss/vite";
import extractorSvelte from "@unocss/extractor-svelte";
import { fileURLToPath } from "node:url";
import { rm } from "node:fs/promises";
import { resolve } from "node:path";

function removeDemoAssetsFromAppBuild(): Plugin {
  let outDir = "dist";

  return {
    name: "remove-demo-assets-from-app-build",
    apply: "build",
    configResolved(config: ResolvedConfig) {
      outDir = config.build.outDir;
    },
    async closeBundle() {
      await Promise.all(
        ["demo-images", "demo-data"].map((dir) =>
          rm(resolve(outDir, dir), { recursive: true, force: true })
        )
      );
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    UnoCSS({
      extractors: [extractorSvelte()],
      /* more options */
    }),
    svelte({
      preprocess: [
        sveltePreprocess({
          typescript: true,
        }),
      ],
    }),
    removeDemoAssetsFromAppBuild(),
  ],

  resolve: {
    preserveSymlinks: true,
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  define: {
    __PUBLIC_DEMO_BUILD__: false,
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
}));
