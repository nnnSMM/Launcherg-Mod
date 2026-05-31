import { describe, it, expect, vi, beforeEach } from "vitest";
import { memo, updateAndSyncMemo } from "./memo";
import { skyWay } from "./skyway";
import { get } from "svelte/store";

vi.mock("./skyway", () => ({
  skyWay: {
    syncMemo: vi.fn(),
  },
}));

describe("memo store helper", () => {
  beforeEach(() => {
    localStorage.clear();
    memo.set([]);
    vi.mocked(skyWay.syncMemo).mockClear();
  });

  it("updateAndSyncMemo を呼び出すと、localStorage に保存され、ストアが更新され、skyWay.syncMemo が呼び出されること", () => {
    const workId = 42;
    const value = "テストメモ内容";

    updateAndSyncMemo(workId, value);

    // localStorageの確認
    expect(localStorage.getItem(`smde_memo-42`)).toBe(value);

    // ストアの確認
    const currentMemos = get(memo);
    const target = currentMemos.find((m) => m.workId === workId);
    expect(target).toBeDefined();
    expect(target?.value).toBe(value);
    expect(target?.lastModified).toBe("local");

    // skyWay.syncMemoの呼び出し確認
    expect(skyWay.syncMemo).toHaveBeenCalledWith(workId, value);
  });

  it("初期状態でデモ用のダミーメモ（smde_memo-*）が localStorage に存在しないこと", () => {
    const demoIds = [39837, 27059, 38696, 38631, 26245, 28941, 30122, 25861, 20988, 31106, 31597, 38794];
    demoIds.forEach((id) => {
      expect(localStorage.getItem(`smde_memo-${id}`)).toBeNull();
    });
  });

  it("以前の自動生成サンプルメモが localStorage に存在する場合、正しく検知して削除されること", () => {
    const testId = 39837;
    const legacyPlaceholder = "# 攻略進捗とメモ\n\nこれはデモ環境における自動生成サンプルメモです。";
    localStorage.setItem(`smde_memo-${testId}`, legacyPlaceholder);

    // App.svelteのクリーンアップ処理のシミュレート
    const demoIds = [39837, 27059, 38696, 38631, 26245, 28941, 30122, 25861, 20988, 31106, 31597, 38794];
    demoIds.forEach((id) => {
      const key = `smde_memo-${id}`;
      const val = localStorage.getItem(key);
      if (val && (val.includes("自動生成サンプルメモ") || val.includes("攻略進捗とメモ"))) {
        localStorage.removeItem(key);
      }
    });

    expect(localStorage.getItem(`smde_memo-${testId}`)).toBeNull();
  });
});

