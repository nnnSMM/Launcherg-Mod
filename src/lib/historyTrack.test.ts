import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { canGoBack, canGoForward, setupHistoryTracker } from "./historyTrack";
import { get } from "svelte/store";

describe("historyTrack utility", () => {
  let cleanup: () => void;

  beforeEach(() => {
    // ストアの初期化
    canGoBack.set(false);
    canGoForward.set(false);
    
    // ハッシュのモック
    window.location.hash = "";
    // ステートのクリア
    window.history.replaceState(null, "");
  });

  afterEach(() => {
    if (cleanup) {
      cleanup();
    }
  });

  it("should initialize stores to false", () => {
    cleanup = setupHistoryTracker();
    expect(get(canGoBack)).toBe(false);
    expect(get(canGoForward)).toBe(false);
  });

  it("should set canGoBack to true when pushing a new state (different path)", () => {
    cleanup = setupHistoryTracker();

    // 異なるパスへの遷移
    window.history.pushState({}, "", "#/works/1");

    expect(get(canGoBack)).toBe(true);
    expect(get(canGoForward)).toBe(false);
  });

  it("should update stores correctly when popstate is triggered with index states", () => {
    cleanup = setupHistoryTracker();

    // 複数のページを遷移
    window.history.pushState({}, "", "#/works/1"); // idx: 1
    window.history.pushState({}, "", "#/works/2"); // idx: 2

    expect(get(canGoBack)).toBe(true);
    expect(get(canGoForward)).toBe(false);

    // popstate イベントを発火させ、idx 1 の画面に戻る
    window.dispatchEvent(new PopStateEvent("popstate", { state: { idx: 1 } }));

    expect(get(canGoBack)).toBe(true);
    expect(get(canGoForward)).toBe(true);

    // さらに戻って初期画面（idx: 0）へ
    window.dispatchEvent(new PopStateEvent("popstate", { state: { idx: 0 } }));

    expect(get(canGoBack)).toBe(false);
    expect(get(canGoForward)).toBe(true);
  });
});
