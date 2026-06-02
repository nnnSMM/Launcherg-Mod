import { marked } from "marked";
import { convertFileSrc } from "@tauri-apps/api/core";

// marked のカスタムレンダラーを定義する
const renderer = new marked.Renderer();
const SAFE_LINK_PROTOCOLS = new Set(["http:", "https:", "mailto:"]);
const SAFE_RESOURCE_PROTOCOLS = new Set([
  "http:",
  "https:",
  "data:",
  "asset:",
  "tauri:",
  "tauri-src:",
  "file:",
]);

const escapeHtmlAttribute = (value: string) =>
  value
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

const isSafeUrl = (value: string, safeProtocols: Set<string>) => {
  const trimmed = value.trim();
  if (!trimmed) {
    return false;
  }
  if (trimmed.startsWith("#")) {
    return true;
  }
  if (/^(asset|file|tauri|tauri-src):/i.test(trimmed)) {
    return safeProtocols.has(`${trimmed.split(":", 1)[0].toLowerCase()}:`);
  }
  try {
    const base =
      typeof window !== "undefined" && window.location?.href
        ? window.location.href
        : "https://launcherg.local/";
    return safeProtocols.has(new URL(trimmed, base).protocol);
  } catch {
    return false;
  }
};

const sanitizeHtml = (html: string): string => {
  if (typeof DOMParser === "undefined") {
    return html;
  }

  const doc = new DOMParser().parseFromString(html, "text/html");
  doc
    .querySelectorAll("script, style, iframe, object, embed, link, meta, base, form")
    .forEach((element) => element.remove());

  doc.body.querySelectorAll("*").forEach((element) => {
    for (const attr of Array.from(element.attributes)) {
      const attrName = attr.name.toLowerCase();
      if (attrName.startsWith("on") || attrName === "srcset" || attrName === "style") {
        element.removeAttribute(attr.name);
        continue;
      }
      if (attrName === "href" && !isSafeUrl(attr.value, SAFE_LINK_PROTOCOLS)) {
        element.removeAttribute(attr.name);
      }
      if (attrName === "src" && !isSafeUrl(attr.value, SAFE_RESOURCE_PROTOCOLS)) {
        element.removeAttribute(attr.name);
      }
    }
  });

  return doc.body.innerHTML;
};

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
  return `<img src="${escapeHtmlAttribute(src)}" alt="${escapeHtmlAttribute(text || "")}" title="${escapeHtmlAttribute(title || "")}" class="max-w-full h-auto rounded-lg shadow-sm" />`;
};

marked.use({ renderer });

export function parseMarkdown(md: string): string {
  if (!md) return "";
  return sanitizeHtml(marked.parse(md) as string);
}
