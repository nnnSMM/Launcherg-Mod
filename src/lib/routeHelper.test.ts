import { describe, it, expect, vi } from "vitest";
import { isWorkDetailRoute, getWorkDetailBgImage, shouldCleanupBgImage } from "./routeHelper";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `tauri-protocol://${path}`,
}));

describe("routeHelper", () => {
  describe("isWorkDetailRoute", () => {
    it("should return true for work or memo detail path", () => {
      expect(isWorkDetailRoute("/works/123")).toBe(true);
      expect(isWorkDetailRoute("/works/abc")).toBe(true);
      expect(isWorkDetailRoute("/memos/123")).toBe(true);
    });

    it("should return false for other paths", () => {
      expect(isWorkDetailRoute("/")).toBe(false);
      expect(isWorkDetailRoute("/settings")).toBe(false);
    });
  });

  describe("getWorkDetailBgImage", () => {
    it("should return url containing thumbnail path when valid", () => {
      const element = {
        thumbnail: "C:\\path\\to\\thumbnail.jpg",
        updatedAt: "2026-05-30T10:00:00.000Z",
      };
      const result = getWorkDetailBgImage(element.thumbnail, element.updatedAt);
      expect(result).toContain("thumbnail.jpg");
      expect(result).toContain("v=2026-05-30T10:00:00.000Z");
    });

    it("should return dummy thumbnail when thumbnail is empty", () => {
      const result = getWorkDetailBgImage("", "2026-05-30T10:00:00.000Z");
      expect(result).toBe("/images/dummy_thumbnail.svg");
    });
  });

  describe("shouldCleanupBgImage", () => {
    it("should return false if next path is work detail", () => {
      expect(shouldCleanupBgImage("/works/123")).toBe(false);
    });

    it("should return false if next path is memo detail", () => {
      expect(shouldCleanupBgImage("/memos/123")).toBe(false);
    });

    it("should return true if next path is home", () => {
      expect(shouldCleanupBgImage("/")).toBe(true);
    });

    it("should return true if next path is settings", () => {
      expect(shouldCleanupBgImage("/settings")).toBe(true);
    });
  });
});
