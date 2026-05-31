import { marked } from "marked";
import { convertFileSrc } from "@tauri-apps/api/core";

// marked のカスタムレンダラーを定義する
const renderer = new marked.Renderer();

renderer.image = (href, title, text) => {
  let src = href || "";
  // Webの画像URLやdataスキーム画像はそのままにする
  const isWebUrl = src.startsWith("http://") || src.startsWith("https://") || src.startsWith("data:");
  if (src && !isWebUrl) {
    try {
      src = convertFileSrc(src);
    } catch (e) {
      console.error("convertFileSrc failed for path: " + src, e);
    }
  }
  return `<img src="${src}" alt="${text || ""}" title="${title || ""}" class="max-w-full h-auto rounded-lg shadow-sm" />`;
};

marked.use({ renderer });

export function parseMarkdown(md: string): string {
  if (!md) return "";
  return marked.parse(md) as string;
}
