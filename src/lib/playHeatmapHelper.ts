import type { CollectionElementDailyPlayTime } from "@/lib/types";

/**
 * 複数ゲームの日別プレイ時間を日付ごとに合算し、日付昇順でソートして返却します。
 * 合算後のデータでは collectionElementId は 0 に統一されます。
 */
export function mergeDailyPlayTimes(
  playTimes: CollectionElementDailyPlayTime[]
): CollectionElementDailyPlayTime[] {
  if (!playTimes || playTimes.length === 0) {
    return [];
  }

  const dateMap = new Map<string, number>();

  for (const time of playTimes) {
    const currentDate = time.playDate;
    const existing = dateMap.get(currentDate) ?? 0;
    dateMap.set(currentDate, existing + time.playTimeSeconds);
  }

  const merged: CollectionElementDailyPlayTime[] = Array.from(dateMap.entries()).map(
    ([playDate, playTimeSeconds]) => ({
      collectionElementId: 0,
      playDate,
      playTimeSeconds,
    })
  );

  // 日付昇順でソート
  return merged.sort((a, b) => a.playDate.localeCompare(b.playDate));
}
