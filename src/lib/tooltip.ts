import { delegate } from "tippy.js";

export function migrateTitle(element: HTMLElement): void {
  const title = element.getAttribute("title");
  if (title) {
    element.setAttribute("data-tippy-content", title);
    element.removeAttribute("title");
  }
}

/**
 * アプリ全体で title 属性を Tippy.js のカスタムツールチップに自動移行するグローバル設定を行います。
 * @returns クリーンアップ関数
 */
export function setupGlobalTooltips(): () => void {
  // MutationObserverを使って、動的に追加された、または変更されたtitle属性を
  // data-tippy-contentに移し替えて、ブラウザ標準のツールチップ表示を防ぐ
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === "attributes" && mutation.attributeName === "title") {
        const target = mutation.target as HTMLElement;
        migrateTitle(target);
      } else if (mutation.type === "childList") {
        mutation.addedNodes.forEach((node) => {
          if (node instanceof HTMLElement) {
            if (node.hasAttribute("title")) {
              migrateTitle(node);
            }
            node.querySelectorAll("[title]").forEach((el) => {
              if (el instanceof HTMLElement) {
                migrateTitle(el);
              }
            });
          }
        });
      }
    }
  });

  observer.observe(document.body, {
    attributes: true,
    childList: true,
    subtree: true,
    attributeFilter: ["title"],
  });

  // 初期化時にすでに存在するtitle属性を移し替える
  document.querySelectorAll("[title]").forEach((el) => {
    if (el instanceof HTMLElement) {
      migrateTitle(el);
    }
  });

  // delegateの対象を[data-tippy-content]にする
  const tp = delegate(document.body, {
    target: "[data-tippy-content]",
    theme: "default",
    arrow: false, // 吹き出しの矢印を非表示にする
    delay: [300, 50],
    content(reference) {
      return reference.getAttribute("data-tippy-content") || "";
    },
    // 動的更新のために、表示される直前にも最新の値を読み込む
    onShow(instance) {
      const content = instance.reference.getAttribute("data-tippy-content");
      if (content) {
        instance.setContent(content);
      }
    },
  });

  return () => {
    observer.disconnect();
    tp.destroy();
  };
}



