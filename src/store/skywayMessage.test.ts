import { afterEach, describe, expect, it, vi } from "vitest";
import { isRemoteMessage, parseRemoteMessage } from "./skywayMessage";

describe("skywayMessage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("accepts valid remote messages", () => {
    expect(isRemoteMessage({ type: "ping" })).toBe(true);
    expect(
      isRemoteMessage({ type: "memo", gameId: 10, text: "memo text" }),
    ).toBe(true);
    expect(isRemoteMessage({ type: "init", gameId: 10 })).toBe(true);
    expect(isRemoteMessage({ type: "library_request" })).toBe(true);
    expect(isRemoteMessage({ type: "control_status_request" })).toBe(true);
    expect(isRemoteMessage({ type: "pause_toggle" })).toBe(true);
    expect(
      isRemoteMessage({
        type: "take_screenshot",
        gameId: 10,
        cursorLine: 3,
        hideText: true,
      }),
    ).toBe(true);
  });

  it("rejects malformed remote messages", () => {
    expect(isRemoteMessage(null)).toBe(false);
    expect(isRemoteMessage({ type: "memo", gameId: 10 })).toBe(false);
    expect(isRemoteMessage({ type: "init", gameId: "10" })).toBe(false);
    expect(
      isRemoteMessage({
        type: "take_screenshot",
        gameId: 10,
        cursorLine: 1.5,
      }),
    ).toBe(false);
    expect(
      isRemoteMessage({
        type: "take_screenshot",
        gameId: 10,
        cursorLine: 1,
        hideText: "yes",
      }),
    ).toBe(false);
    expect(isRemoteMessage({ type: "unknown" })).toBe(false);
  });

  it("parses a valid remote message", () => {
    const message = parseRemoteMessage(
      JSON.stringify({ type: "memo", gameId: 10, text: "memo text" }),
    );

    expect(message).toEqual({ type: "memo", gameId: 10, text: "memo text" });
  });

  it("returns null for invalid JSON without throwing", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});

    expect(parseRemoteMessage("{ invalid json")).toBeNull();

    expect(warn).toHaveBeenCalled();
  });

  it("returns null for an unexpected payload without throwing", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});

    expect(parseRemoteMessage(JSON.stringify({ type: "unknown" }))).toBeNull();

    expect(warn).toHaveBeenCalled();
  });
});
