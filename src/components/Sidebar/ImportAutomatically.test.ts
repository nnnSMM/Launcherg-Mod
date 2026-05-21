import { describe, it, expect, beforeEach, vi } from "vitest";
import ImportAutomatically from "./ImportAutomatically.svelte";

describe("ImportAutomatically - デモビルド環境", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    vi.stubEnv("BASE_URL", "./");
    class MockIntersectionObserver {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
    vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);
  });

  it("デモビルドの時も通常ビルドと同じUIが表示され、各入力やボタンが非活性になっていること", async () => {
    const target = document.body;
    new ImportAutomatically({
      target,
      props: {
        isOpen: true,
      },
    });

    // 警告メッセージが表示されず、通常フォームが表示されることを確認
    const heading = target.querySelector(".text-h4")?.textContent;
    expect(heading).toContain("自動追加するフォルダ");

    const inputPath = target.querySelector('input[placeholder*="Program Files"]') as HTMLInputElement;
    expect(inputPath).not.toBeNull();

    // 以下のテストは実装前（未実装）のため失敗するはず (Red)
    expect(inputPath.disabled).toBe(true);
    expect(inputPath.value).toBe("C:\\Program Files (x86)\\demo-games");
  });
});
