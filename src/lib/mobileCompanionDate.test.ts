import { describe, expect, it } from "vitest";
import {
  safeFormatLastPlay,
  safeFormatSyncTime,
} from "./mobileCompanionDate";

describe("mobileCompanionDate helper functions", () => {
  describe("safeFormatLastPlay", () => {
    it("returns '未プレイ' for null or undefined", () => {
      expect(safeFormatLastPlay(null)).toBe("未プレイ");
      expect(safeFormatLastPlay(undefined)).toBe("未プレイ");
    });

    it("returns '未プレイ' for empty string", () => {
      expect(safeFormatLastPlay("")).toBe("未プレイ");
    });

    it("returns '未プレイ' for invalid date strings", () => {
      expect(safeFormatLastPlay("invalid-date-format")).toBe("未プレイ");
      expect(safeFormatLastPlay("null")).toBe("未プレイ");
      expect(safeFormatLastPlay("undefined")).toBe("未プレイ");
    });

    it("formats valid dates to MM/DD format", () => {
      const dateStr = "2026-06-06T12:00:00Z";
      const formatted = safeFormatLastPlay(dateStr);
      // toLocaleDateString の挙動（日本のロケール）で "06/06" もしくは "6/6" になるはず
      expect(formatted).toMatch(/^(0?6\/0?6|0?6-0?6)$/);
    });
  });

  describe("safeFormatSyncTime", () => {
    it("returns '未同期' for null or undefined", () => {
      expect(safeFormatSyncTime(null)).toBe("未同期");
      expect(safeFormatSyncTime(undefined)).toBe("未同期");
    });

    it("returns '未同期' for empty string", () => {
      expect(safeFormatSyncTime("")).toBe("未同期");
    });

    it("returns '未同期' for invalid date strings", () => {
      expect(safeFormatSyncTime("invalid-date-format")).toBe("未同期");
      expect(safeFormatSyncTime("null")).toBe("未同期");
      expect(safeFormatSyncTime("undefined")).toBe("未同期");
    });

    it("formats valid times to HH:MM format", () => {
      const dateStr = "2026-06-06T12:34:56.000Z";
      const formatted = safeFormatSyncTime(dateStr);
      // タイムゾーンによる時差（日本なら +9時間で 21:34）を考慮してフォーマットされる
      expect(formatted).toMatch(/^\d{2}:\d{2}$/);
    });
  });
});
