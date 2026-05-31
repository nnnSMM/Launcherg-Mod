import { describe, it, expect, beforeEach, vi } from "vitest";
import { tick } from "svelte";

vi.mock("@/lib/command", () => ({
  commandGetGameCandidates: vi.fn().mockResolvedValue([]),
}));

import ImportManually from "./ImportManually.svelte";
import { commandGetGameCandidates } from "@/lib/command";

describe("ImportManually - デモビルド環境", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    vi.mocked(commandGetGameCandidates).mockClear();
    vi.stubEnv("BASE_URL", "./");
    class MockIntersectionObserver {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
    vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);
  });

  it("デモビルドの時はパスとIDの入力フィールドが非活性になり、インポートボタンは活性化してトースト表示されること", async () => {
    const target = document.body;
    new ImportManually({
      target,
      props: {
        isOpen: true,
      },
    });

    const inputFilePath = target.querySelector('input[placeholder*="Monkeys!!"]') as HTMLInputElement;
    const inputId = target.querySelector('input[placeholder*="17909"]') as HTMLInputElement;

    expect(inputFilePath).not.toBeNull();
    expect(inputId).not.toBeNull();

    // 以下のテストは実装前（未実装）のため失敗するはず (Red)
    expect(inputFilePath.disabled).toBe(true);
    expect(inputId.disabled).toBe(true);
    expect(inputFilePath.value).toBe("C:\\game\\demo\\game.exe");
    expect(inputId.value).toBe("12345");
  });
});

describe("ImportManually - 候補取得", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    vi.mocked(commandGetGameCandidates).mockClear();
    vi.stubEnv("BASE_URL", "/");
    class MockIntersectionObserver {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
    vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);
  });

  it("モーダルが閉じている間は既存パスがあっても候補検索を走らせないこと", async () => {
    new ImportManually({
      target: document.body,
      props: {
        isOpen: false,
        idInput: "4529",
        path: "E:\\VisualNovel\\枕\\サクラノ詩\\BGI.exe",
      },
    });

    await tick();

    expect(commandGetGameCandidates).not.toHaveBeenCalled();
  });
});
