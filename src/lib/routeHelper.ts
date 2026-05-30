import { convertFileSrc } from "@tauri-apps/api/core";

export function isWorkDetailRoute(path: string): boolean {
  return path.startsWith("/works/") || path.startsWith("/memos/");
}

export function getWorkDetailBgImage(
  thumbnail: string | null | undefined,
  updatedAt: string | null | undefined
): string {
  if (!thumbnail || thumbnail.trim() === "") {
    return "/images/dummy_thumbnail.svg";
  }
  return `${convertFileSrc(thumbnail)}?v=${updatedAt ?? ""}`;
}

export function shouldCleanupBgImage(nextPath: string): boolean {
  return !isWorkDetailRoute(nextPath);
}
