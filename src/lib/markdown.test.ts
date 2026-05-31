import { describe, it, expect, vi } from "vitest";
import { parseMarkdown } from "./markdown";

// @tauri-apps/api/core の convertFileSrc をモックする
vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `tauri-src://${path}`,
}));

describe("parseMarkdown", () => {
  it("通常のマークダウンがHTMLに正しく変換されること", () => {
    const md = "# 見出し\n\n- リスト1\n- リスト2\n\nこれは**太字**です。";
    const html = parseMarkdown(md);
    expect(html).toContain('<h1 id="見出し">見出し</h1>');
    expect(html).toContain("<ul>");
    expect(html).toContain("<li>リスト1</li>");
    expect(html).toContain("<strong>太字</strong>");
  });

  it("Web画像パス（http, https）はそのまま出力されること", () => {
    const md = "![web画像](https://example.com/image.png)";
    const html = parseMarkdown(md);
    expect(html).toContain('src="https://example.com/image.png"');
    expect(html).toContain('alt="web画像"');
  });

  it("dataスキーム画像はそのまま出力されること", () => {
    const md = "![data画像](data:image/png;base64,iVBORw0KGgo=)";
    const html = parseMarkdown(md);
    expect(html).toContain('src="data:image/png;base64,iVBORw0KGgo="');
  });

  it("ローカル画像パスは convertFileSrc が適用されたパスに変換されること", () => {
    const md = "![ローカル画像](C:\\path\\to\\local\\image.png)";
    const html = parseMarkdown(md);
    expect(html).toContain('src="tauri-src://C:\\path\\to\\local\\image.png"');
    expect(html).toContain('alt="ローカル画像"');
  });
});
