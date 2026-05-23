import { writable } from "svelte/store";

export const canGoBack = writable(false);
export const canGoForward = writable(false);

/**
 * グローバル履歴トラッキングのセットアップを行います。
 * 本番環境(Tauri/WebView2)ではモダンな Navigation API を使用し、
 * テスト環境(jsdom)などの未サポート環境ではメモリスタック方式へフォールバックします。
 * @returns クリーンアップ関数
 */
export function setupHistoryTracker(): () => void {
  if (typeof window === "undefined") return () => {};

  const nav = (window as any).navigation;

  // ==========================================
  // 1. Navigation API を使用した本番向けの実装
  // ==========================================
  if (nav && typeof nav.addEventListener === "function") {
    const syncStores = () => {
      canGoBack.set(nav.canGoBack);
      canGoForward.set(nav.canGoForward);
    };

    // 初期状態の同期
    syncStores();

    // 遷移が発生し履歴エントリが変更された際にあらゆる遷移タイプで発火
    nav.addEventListener("currententrychange", syncStores);

    return () => {
      nav.removeEventListener("currententrychange", syncStores);
    };
  }

  // ==========================================
  // 2. 未サポート環境 (Vitest/jsdom 等) のための history.state.idx 同期実装
  // ==========================================
  
  // メモリ上で追跡するインデックスと最大インデックス
  let currentIdx = 0;
  let maxIdx = 0;

  const updateStoresFallback = () => {
    canGoBack.set(currentIdx > 0);
    canGoForward.set(currentIdx < maxIdx);
  };

  // 初期化時に state.idx がなければ 0 をセットする
  let initialState = window.history.state;
  if (!initialState || typeof initialState.idx !== "number") {
    window.history.replaceState({ ...(initialState || {}), idx: 0 }, "");
    currentIdx = 0;
    maxIdx = 0;
  } else {
    currentIdx = initialState.idx;
    maxIdx = currentIdx;
  }

  updateStoresFallback();

  // 元のメソッドを退避
  const origPushState = window.history.pushState;
  const origReplaceState = window.history.replaceState;

  // pushStateのラップ
  window.history.pushState = function (state, title, url) {
    // pushされたら、新しいインデックスは現在のインデックス + 1
    currentIdx++;
    maxIdx = currentIdx;
    
    const nextState = { ...(state || {}), idx: currentIdx };
    origPushState.call(window.history, nextState, title, url);
    updateStoresFallback();
  };

  // replaceStateのラップ
  window.history.replaceState = function (state, title, url) {
    // replaceされたら、インデックスは変わらず、そのエントリに現在のインデックスを上書きする
    const nextState = { ...(state || {}), idx: currentIdx };
    origReplaceState.call(window.history, nextState, title, url);
    updateStoresFallback();
  };

  // popstate / hashchange (ブラウザの進む戻るボタン、または location.hash 代入による hashchange)
  const handlePopState = (event: PopStateEvent | HashChangeEvent) => {
    // PopStateEvent の場合は event.state を優先して参照
    const state = (event as any).state || window.history.state;
    
    if (state && typeof state.idx === "number") {
      if (state.idx !== currentIdx) {
        // state.idx が存在し、現在の currentIdx と異なる場合は、履歴の戻る・進む（pop）が発生した
        currentIdx = state.idx;
        updateStoresFallback();
        return;
      }
    }
    
    // state が null であるか、idx が currentIdx と同じ場合（＝ location.hash 代入などの新規のハッシュ遷移）
    // この場合、ブラウザが新しい履歴エントリを作成しているため、新規遷移（push）としてインデックスを刻みます。
    currentIdx++;
    maxIdx = currentIdx;
    
    // 新しい履歴エントリに idx を刻む
    const nextState = { ...(state || {}), idx: currentIdx };
    origReplaceState.call(window.history, nextState, "");
    updateStoresFallback();
  };

  window.addEventListener("popstate", handlePopState);
  window.addEventListener("hashchange", handlePopState as any);

  // クリーンアップ関数を返す
  return () => {
    window.history.pushState = origPushState;
    window.history.replaceState = origReplaceState;
    window.removeEventListener("popstate", handlePopState);
    window.removeEventListener("hashchange", handlePopState as any);
  };
}
