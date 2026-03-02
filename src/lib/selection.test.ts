import { describe, it, expect } from "vitest";
import { selectAll, deselectAll, toggleAll } from "./selection";

describe("selection logic", () => {
  describe("selectAll", () => {
    it("現在のitemsのすべてのidが追加される。既に選択済みのものやitemsにないものも保持される", () => {
      const items = [{ id: 10 }, { id: 20 }];
      const currentSelection = new Set<number>([99, 10]);
      const res = selectAll(items, currentSelection);
      expect(res.size).toBe(3);
      expect(res.has(10)).toBe(true);
      expect(res.has(20)).toBe(true);
      expect(res.has(99)).toBe(true);
    });
  });

  describe("deselectAll", () => {
    it("現在のitemsのidだけが選択解除される。itemsにない選択済み要素は維持される", () => {
      const items = [{ id: 10 }, { id: 20 }];
      const currentSelection = new Set<number>([99, 10, 20]);
      const res = deselectAll(items, currentSelection);
      expect(res.size).toBe(1);
      expect(res.has(99)).toBe(true);
      expect(res.has(10)).toBe(false);
      expect(res.has(20)).toBe(false);
    });
  });

  describe("toggleAll", () => {
    it("現在のitemsのidについて、選択されているものは解除し、未選択のものは選択する。itemsにない選択済み要素は維持される", () => {
      const items = [{ id: 10 }, { id: 20 }, { id: 30 }];
      const currentSelection = new Set<number>([10, 20, 99]);
      const res = toggleAll(items, currentSelection);
      // 10, 20 are deselected. 30 is newly selected. 99 is kept.
      expect(res.has(10)).toBe(false);
      expect(res.has(20)).toBe(false);
      expect(res.has(30)).toBe(true);
      expect(res.has(99)).toBe(true);
    });
  });
});
