export function isWorkDetailRoute(path: string): boolean {
  return path.startsWith("/works/") || path.startsWith("/memos/");
}

export async function getWorkDetailBgImage(
  thumbnail: string | null | undefined,
  updatedAt: string | null | undefined
): Promise<string> {
  if (!thumbnail || thumbnail.trim() === "") {
    return "/images/dummy_thumbnail.svg";
  }
  const { convertFileSrc } = await import("@tauri-apps/api/core");
  return `${convertFileSrc(thumbnail)}?v=${updatedAt ?? ""}`;
}

export function shouldCleanupBgImage(nextPath: string): boolean {
  return !isWorkDetailRoute(nextPath);
}
