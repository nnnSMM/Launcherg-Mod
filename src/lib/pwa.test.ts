import { afterEach, describe, expect, it, vi } from "vitest";
import { isStandalonePwa } from "./pwa";

const setMatchMedia = (matches: boolean) => {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: vi.fn().mockReturnValue({ matches }),
  });
};

const setNavigatorStandalone = (standalone: boolean | undefined) => {
  Object.defineProperty(navigator, "standalone", {
    configurable: true,
    value: standalone,
  });
};

describe("isStandalonePwa", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      value: undefined,
    });
    setNavigatorStandalone(undefined);
  });

  it("detects installed PWA windows through display-mode", () => {
    setMatchMedia(true);
    setNavigatorStandalone(false);

    expect(isStandalonePwa()).toBe(true);
  });

  it("detects iOS home screen apps through navigator.standalone", () => {
    setMatchMedia(false);
    setNavigatorStandalone(true);

    expect(isStandalonePwa()).toBe(true);
  });

  it("does not treat a normal browser tab as standalone", () => {
    setMatchMedia(false);
    setNavigatorStandalone(false);

    expect(isStandalonePwa()).toBe(false);
  });
});
