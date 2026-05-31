import { describe, it, expect, beforeEach } from "vitest";
import { tick } from "svelte";
import APopoverTestHelper from "./APopoverTestHelper.svelte";

describe("APopover", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("renders the glass menu surface when opened", async () => {
    const target = document.body;
    new APopoverTestHelper({
      target,
    });
    await tick();

    const triggerBtn = target.querySelector("#trigger-btn") as HTMLButtonElement | null;
    expect(triggerBtn).not.toBeNull();

    triggerBtn?.click();
    await tick();
    await tick();

    const popoverContent = target.querySelector("#popover-content");
    expect(popoverContent).not.toBeNull();

    const container = popoverContent?.closest(".fixed") as HTMLDivElement | null;
    expect(container).not.toBeNull();
    expect(container?.className).toContain("glass-menu-surface");
  });

  it("keeps the popover open when a pointer drag starts inside and ends outside", async () => {
    const target = document.body;
    new APopoverTestHelper({
      target,
    });
    await tick();

    const triggerBtn = target.querySelector("#trigger-btn") as HTMLButtonElement | null;
    expect(triggerBtn).not.toBeNull();
    triggerBtn?.click();
    await tick();
    await tick();

    const popoverContent = target.querySelector("#popover-content") as HTMLDivElement | null;
    expect(popoverContent).not.toBeNull();

    popoverContent?.dispatchEvent(
      new Event("pointerdown", { bubbles: true, cancelable: true }),
    );
    document.body.dispatchEvent(
      new MouseEvent("click", { bubbles: true, cancelable: true }),
    );
    await tick();

    expect(target.querySelector("#popover-content")).not.toBeNull();
  });

});
