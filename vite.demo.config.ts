import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import sveltePreprocess from "svelte-preprocess";
import UnoCSS from "unocss/vite";
import { fileURLToPath } from "node:url";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const defaultSiteUrl = "https://nnnsmm.github.io/Launcherg-Mod/";

const getSiteUrl = () => {
  const rawSiteUrl =
    process.env.PUBLIC_SITE_URL || process.env.SITE_URL || defaultSiteUrl;
  return rawSiteUrl.endsWith("/") ? rawSiteUrl : `${rawSiteUrl}/`;
};

const siteUrl = getSiteUrl();
const demoBuildId = (
  process.env.DEMO_BUILD_ID ||
  process.env.GITHUB_SHA?.slice(0, 12) ||
  Date.now().toString(36)
).replace(/[^a-zA-Z0-9_-]/g, "");

const escapeXml = (value: string) =>
  value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

const createSeoFilesPlugin = () => ({
  name: "create-demo-seo-files",
  async closeBundle() {
    const outDir = resolve("docs/demo");
    await mkdir(outDir, { recursive: true });
    await writeFile(
      resolve(outDir, "robots.txt"),
      [
        "User-agent: *",
        "Allow: /",
        "",
        `Sitemap: ${siteUrl}sitemap.xml`,
        "",
      ].join("\n"),
    );
    await writeFile(
      resolve(outDir, "sitemap.xml"),
      [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
        "  <url>",
        `    <loc>${escapeXml(siteUrl)}</loc>`,
        "  </url>",
        "</urlset>",
        "",
      ].join("\n"),
    );
  },
});

const createPwaAssetManifestPlugin = () => ({
  name: "create-demo-pwa-asset-manifest",
  async closeBundle() {
    const outDir = resolve("docs/demo");
    const assetsDir = resolve(outDir, "assets");
    const swPath = resolve(outDir, "sw.js");

    let serviceWorker: string;
    try {
      serviceWorker = await readFile(swPath, "utf-8");
    } catch {
      return;
    }

    const entries = await readdir(assetsDir, { withFileTypes: true }).catch(
      () => [],
    );
    const assets = entries
      .filter((entry) => entry.isFile() && /\.(js|css)$/.test(entry.name))
      .map((entry) => `./assets/${entry.name}`)
      .sort();

    const injectedAssets = `self.__LAUNCHERG_PWA_ASSETS__ = ${JSON.stringify(
      assets,
      null,
      2,
    )};\n`;
    await writeFile(swPath, `${injectedAssets}${serviceWorker}`);
  },
});

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
    createSeoFilesPlugin(),
    createPwaAssetManifestPlugin(),
  ],

  resolve: {
    preserveSymlinks: true,
    alias: {
      "@/store/works": fileURLToPath(new URL("./src/mock/works.ts", import.meta.url)),
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "@tauri-apps/api/app": fileURLToPath(new URL("./src/mock/tauri-app.ts", import.meta.url)),
      "@tauri-apps/api/core": fileURLToPath(new URL("./src/mock/tauri-core.ts", import.meta.url)),
      "@tauri-apps/api/window": fileURLToPath(new URL("./src/mock/tauri-window.ts", import.meta.url)),
      "@tauri-apps/api/event": fileURLToPath(new URL("./src/mock/tauri-event.ts", import.meta.url)),
      "@tauri-apps/plugin-dialog": fileURLToPath(new URL("./src/mock/tauri-dialog.ts", import.meta.url)),
      "@tauri-apps/plugin-shell": fileURLToPath(new URL("./src/mock/tauri-shell.ts", import.meta.url)),
      "@tauri-apps/plugin-process": fileURLToPath(new URL("./src/mock/tauri-process.ts", import.meta.url)),
      "@tauri-apps/plugin-updater": fileURLToPath(new URL("./src/mock/tauri-updater.ts", import.meta.url)),
      "@tauri-apps/plugin-fs": fileURLToPath(new URL("./src/mock/tauri-fs.ts", import.meta.url)),
      "@tauri-apps/plugin-clipboard-manager": fileURLToPath(new URL("./src/mock/tauri-clipboard.ts", import.meta.url)),
      "@tauri-apps/plugin-http": fileURLToPath(new URL("./src/mock/tauri-http.ts", import.meta.url)),
    },
  },

  define: {
    __PUBLIC_DEMO_BUILD__: true,
  },

  build: {
    outDir: "docs/demo",
    emptyOutDir: true,
    chunkSizeWarningLimit: 7000,
    rollupOptions: {
      output: {
        entryFileNames: `assets/[name]-[hash]-${demoBuildId}.js`,
        chunkFileNames: `assets/[name]-[hash]-${demoBuildId}.js`,
        assetFileNames: `assets/[name]-[hash]-${demoBuildId}[extname]`,
      },
    },
  },
}));
