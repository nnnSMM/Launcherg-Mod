import { describe, it, expect, beforeEach } from "vitest";
import ContextMenuComponent from "./ContextMenu.svelte";

describe("ContextMenu", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("コンテキストメニューコンテナがグラス仕様（bg-bg-secondary/70, backdrop-blur-md, shadow-lg）を持っていること", () => {
    const target = document.body;
    new ContextMenuComponent({
      target,
      props: {
        x: 100,
        y: 100,
        options: [{ label: "テストオプション", onSelect: () => {} }],
      },
    });

    const menuEl = target.querySelector('[role="menu"]');
    expect(menuEl).not.toBeNull();
    if (menuEl) {
      expect(menuEl.className).toContain("glass-menu-surface");
    }

    const buttonEl = target.querySelector("button");
    expect(buttonEl).not.toBeNull();
    if (buttonEl) {
      const classes = buttonEl.className.split(" ");
      expect(classes).toContain("bg-transparent");
      expect(classes).not.toContain("bg-bg-tertiary");
    }
  });
});
