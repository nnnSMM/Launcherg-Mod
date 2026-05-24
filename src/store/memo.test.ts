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
});
