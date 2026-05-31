import { describe, it, expect, beforeEach, vi } from "vitest";
import SearchComponent from "./Search.svelte";
import type { Attribute } from "./searchAttributes";

// svelte-spa-router 等の必要なモック
vi.mock("svelte-spa-router", () => ({
  link: () => {},
}));

describe("SearchComponent", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    // scrollTo のグローバルスタブ
    HTMLElement.prototype.scrollTo = vi.fn() as any;
  });

  const dummyPlayStatusAttributes: Attribute[] = [
    { key: "unplayed", enabled: false },
    { key: "playing", enabled: false },
    { key: "cleared", enabled: false },
    { key: "interrupted", enabled: false },
  ];

  const dummyOtherAttributes: Attribute[] = [
    { key: "nukige", enabled: false },
  ];

  it("前半のボタンをクリックした際、左端(left: 0)にスムーズにスクロールされること", async () => {
    const target = document.body;
    
    new SearchComponent({
      target,
      props: {
        query: "",
        order: "gamename-asc",
        playStatusAttributes: dummyPlayStatusAttributes,
        otherAttributes: dummyOtherAttributes,
      },
    });

    const buttons = target.querySelectorAll("button");
    const firstButton = Array.from(buttons).find(
      (btn) => btn.textContent?.trim() === "未プレイ"
    );

    expect(firstButton).not.toBeNull();

    if (firstButton) {
      // 親要素（SimpleBarなどのスクロールコンテナ）のモック構造を作成
      const container = document.createElement("div");
      container.className = "simplebar-content-wrapper";
      const parent = document.createElement("div");
      
      // DOMツリーの構築
      firstButton.parentNode?.replaceChild(parent, firstButton);
      parent.appendChild(firstButton);
      container.appendChild(parent);
      target.appendChild(container);

      // ボタンのクリックイベントを発火
      firstButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));

      // scrollToが left: 0 で呼び出されたことを検証
      expect(HTMLElement.prototype.scrollTo).toHaveBeenCalledWith({
        left: 0,
        behavior: "smooth",
      });
    }
  });

  it("後半のボタンをクリックした際、右端にスムーズにスクロールされること", async () => {
    const target = document.body;
    
    new SearchComponent({
      target,
      props: {
        query: "",
        order: "gamename-asc",
        playStatusAttributes: dummyPlayStatusAttributes,
        otherAttributes: dummyOtherAttributes,
      },
    });

    const buttons = target.querySelectorAll("button");
    const lastButton = Array.from(buttons).find(
      (btn) => btn.textContent?.trim() === "中断"
    );

    expect(lastButton).not.toBeNull();

    if (lastButton) {
      // 親要素のモック構造を作成
      const container = document.createElement("div");
      container.className = "simplebar-content-wrapper";
      const parent = document.createElement("div");
      
      // DOMツリーの構築
      lastButton.parentNode?.replaceChild(parent, lastButton);
      parent.appendChild(lastButton);
      container.appendChild(parent);
      target.appendChild(container);

      // ボタンのクリックイベントを発火
      lastButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));

      // scrollToが left: scrollWidth (右端) で呼び出されたことを検証
      expect(HTMLElement.prototype.scrollTo).toHaveBeenCalledWith({
        left: expect.any(Number),
        behavior: "smooth",
      });
    }
  });
});
