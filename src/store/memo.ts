import { writable } from "svelte/store";
import { skyWay } from "./skyway";

export const memo = writable<
  { workId: number; value: string; lastModified: "remote" | "local" }[]
>([]);

export function updateAndSyncMemo(workId: number, value: string) {
  localStorage.setItem(`smde_memo-${workId}`, value);
  memo.update((memos) => {
    const filtered = memos.filter((m) => m.workId !== workId);
    return [{ workId, value, lastModified: "local" }, ...filtered];
  });
  skyWay.syncMemo(workId, value);
}
