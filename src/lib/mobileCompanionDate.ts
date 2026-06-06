/**
 * モバイルコンパニオン用の日付フォーマットヘルパー。
 * 不正な日付文字列による RangeError クラッシュを防ぐ。
 */

const isValidDate = (date: Date): boolean =>
  !isNaN(date.getTime());

export const safeFormatLastPlay = (value: string | null | undefined): string => {
  if (!value) return "未プレイ";
  const date = new Date(value);
  if (!isValidDate(date)) return "未プレイ";
  return date.toLocaleDateString("ja-JP", {
    month: "2-digit",
    day: "2-digit",
  });
};

export const safeFormatSyncTime = (value: string | null | undefined): string => {
  if (!value) return "未同期";
  const date = new Date(value);
  if (!isValidDate(date)) return "未同期";
  return date.toLocaleTimeString("ja-JP", {
    hour: "2-digit",
    minute: "2-digit",
  });
};
