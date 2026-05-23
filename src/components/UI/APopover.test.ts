import { describe, it, expect, beforeEach } from "vitest";
import { tick } from "svelte";
import APopoverTestHelper from "./APopoverTestHelper.svelte";

describe("APopover", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("ボタン押下後に表示されるポップオーバーコンテナがグラス仕様（bg-bg-secondary/70, backdrop-blur-md, shadow-lg）を持っていること", async () => {
    const target = document.body;
    new APopoverTestHelper({
      target,
    });

    const triggerBtn = target.querySelector("#trigger-btn") as HTMLButtonElement | null;
    expect(triggerBtn).not.toBeNull();

    if (triggerBtn) {
      triggerBtn.click();
      await tick(); // Svelteの描画更新を待つ
      await tick(); // HeadlessUIの非同期処理やアニメーションのトリガーを考慮して2回呼ぶ

      // ポップオーバーの内容が表示されている要素の親コンテナ（ポップオーバーパネルを包むdiv）を取得する
      // クラス名に absolute や z-10000 を含む要素を探す
      const popoverContent = target.querySelector("#popover-content");
      expect(popoverContent).not.toBeNull();

      if (popoverContent) {
        // ポップオーバーの中身を包んでいる absolute z-10000 の div コンテナを探す
        const container = popoverContent.closest(".fixed") as HTMLDivElement | null;
        expect(container).not.toBeNull();

        if (container) {
          expect(container.className).toContain("glass-menu-surface");
        }
      }
    }
  });
});
