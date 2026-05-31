import { describe, it, expect } from "vitest";
import { migrateTitle, setupGlobalTooltips } from "./tooltip";

describe("tooltip utility", () => {
  describe("migrateTitle", () => {
    it("should migrate title attribute to data-tippy-content and remove title", () => {
      const element = document.createElement("div");
      element.setAttribute("title", "テスト用ツールチップ");

      migrateTitle(element);

      expect(element.getAttribute("title")).toBeNull();
      expect(element.getAttribute("data-tippy-content")).toBe("テスト用ツールチップ");
    });

    it("should do nothing if element has no title attribute", () => {
      const element = document.createElement("div");

      migrateTitle(element);

      expect(element.getAttribute("title")).toBeNull();
      expect(element.getAttribute("data-tippy-content")).toBeNull();
    });
  });

  describe("setupGlobalTooltips", () => {
    it("should migrate existing title attributes on setup", () => {
      const container = document.createElement("div");
      const child = document.createElement("span");
      child.setAttribute("title", "既存のツールチップ");
      container.appendChild(child);
      document.body.appendChild(container);

      const cleanup = setupGlobalTooltips();

      try {
        expect(child.getAttribute("title")).toBeNull();
        expect(child.getAttribute("data-tippy-content")).toBe("既存のツールチップ");
      } finally {
        cleanup();
        document.body.removeChild(container);
      }
    });

    it("should migrate dynamically added title attributes", async () => {
      const container = document.createElement("div");
      document.body.appendChild(container);

      const cleanup = setupGlobalTooltips();

      try {
        const child = document.createElement("span");
        child.setAttribute("title", "動的追加のツールチップ");
        container.appendChild(child);

        // MutationObserverが発火するのを少し待つ
        await new Promise((resolve) => setTimeout(resolve, 50));

        expect(child.getAttribute("title")).toBeNull();
        expect(child.getAttribute("data-tippy-content")).toBe("動的追加のツールチップ");
      } finally {
        cleanup();
        document.body.removeChild(container);
      }
    });

    it("should migrate updated title attributes", async () => {
      const container = document.createElement("div");
      const child = document.createElement("span");
      container.appendChild(child);
      document.body.appendChild(container);

      const cleanup = setupGlobalTooltips();

      try {
        child.setAttribute("title", "更新されたツールチップ");

        // MutationObserverが発火するのを少し待つ
        await new Promise((resolve) => setTimeout(resolve, 50));

        expect(child.getAttribute("title")).toBeNull();
        expect(child.getAttribute("data-tippy-content")).toBe("更新されたツールチップ");
      } finally {
        cleanup();
        document.body.removeChild(container);
      }
    });
  });
});

