import { createLocalStorageWritable, localStorageWritable } from "@/lib/utils";
import { push, type RouteLoadedEvent } from "svelte-spa-router";

export type Tab = {
  id: number;
  workId: number;
  type: "works" | "memos";
  scrollTo: number;
  title: string;
};

const isValidTabType = (src: string): src is "works" | "memos" => {
  return src === "works" || src === "memos";
};
const createTabs = () => {
  const [tabs, getTabs] = createLocalStorageWritable<Tab[]>("tabs", [
    { id: 0, workId: 7402, type: "works", scrollTo: 0, title: "G線上の魔王" },
    {
      id: 2,
      workId: 21228,
      type: "memos",
      scrollTo: 0,
      title: "メモ - G線上の魔王",
    },
    { id: 3, workId: 20460, type: "works", scrollTo: 0, title: "G線上の魔王" },
    {
      id: 4,
      workId: 21531,
      type: "memos",
      scrollTo: 0,
      title: "メモ - G線上の魔王",
    },
  ]);

  const [selected, getSelected] = createLocalStorageWritable("tab-selected", 0);

  const routeLoaded = (event: RouteLoadedEvent) => {
    const isHome = event.detail.location === "/";
    if (isHome) {
      selected.set(-1);
      return;
    }

    const params = event.detail.params;
    if (!params) {
      console.error("params is null (not home)");
      return;
    }
    const id = +params["id"];
    if (!id || isNaN(id)) {
      console.error("params[id] is undefined (not home)");
      return;
    }

    const tabType = event.detail.location.split("/")[1];
    if (!isValidTabType(tabType)) {
      console.error("tabType is invalid (not home)");
      return;
    }

    const tabIndex = getTabs().findIndex(
      (v) => v.workId === id && v.type === tabType
    );
    if (tabIndex === -1) {
      const searchParams = new URLSearchParams(event.detail.querystring);
      const gamename = searchParams.get("gamename");
      if (!gamename) {
        console.error("tabs にないのに gamename の queryParam がない");
        return;
      }
      let title = gamename;
      if (tabType === "memos") {
        title = `メモ - ${title}`;
      }
      const newTab: Tab = {
        id: new Date().getTime(),
        type: tabType,
        workId: id,
        scrollTo: 0,
        title,
      };
      tabs.update((v) => {
        return [...v, newTab];
      });
      const newSelected = getTabs().length - 1;
      selected.set(newSelected);
    } else {
      selected.set(tabIndex);
    }
  };
  const deleteTab = (id: number) => {
    const deleteIndex = getTabs().findIndex((v) => v.id === id);
    const currentIndex = getSelected();

    const isCurrentTab = deleteIndex === currentIndex;
    const isDeletePrevTab = deleteIndex < currentIndex;
    const isRightestTab = deleteIndex === getTabs().length - 1;

    tabs.update((v) => {
      const newTabs = v.filter((tab) => tab.id !== id);
      if (newTabs.length === 0) {
        push("/");
      }
      return newTabs;
    });

    if (isRightestTab && getTabs().length === 0) {
      return;
    }

    if (isCurrentTab) {
      const newIndex = isRightestTab ? currentIndex - 1 : currentIndex;
      if (newIndex < 0 && getTabs().length > 0) { // 全削除後に1つだけ残った場合など
          const nextTab = getTabs()[0];
          push(`/${nextTab.type}/${nextTab.workId}`);
          selected.set(0); // selectedも更新
      } else if (getTabs().length > 0) {
          const nextTab = getTabs()[newIndex];
          push(`/${nextTab.type}/${nextTab.workId}`);
          // selected は routeLoaded で更新されるはず
      }
      return;
    }

    if (isDeletePrevTab) {
      selected.update((v) => v - 1);
      return;
    }
  };

  const initialize = () => {
    const _tabs = getTabs();
    const index = getSelected();
    if (_tabs.length === 0 && index === -1) { // 初期状態でタブが空、selectedも-1ならホーム
        push("/");
        return;
    }
    if (_tabs.length - 1 < index || index < 0) {
      console.warn("Invalid selected index, redirecting to home.", { // console.error から warn に変更
        tabs: getTabs(),
        selected: getSelected(),
      });
      selected.set(-1);
      push("/");
      return;
    }
    const tab = _tabs[index];
    push(`/${tab.type}/${tab.workId}`);
  };

  const getSelectedTab = () => {
    const selIndex = getSelected();
    const currentTabs = getTabs();
    if (selIndex >= 0 && selIndex < currentTabs.length) {
        return currentTabs[selIndex];
    }
    return undefined; // 範囲外なら undefined を返す
  };

  // --- ここから追加 ---
  const reorderTabs = (oldIndex: number, newIndex: number) => {
    tabs.update(currentTabs => {
      const itemToMove = currentTabs[oldIndex];
      const remainingItems = currentTabs.filter((_, index) => index !== oldIndex);
      const reorderedTabs = [
        ...remainingItems.slice(0, newIndex),
        itemToMove,
        ...remainingItems.slice(newIndex)
      ];

      // アクティブなタブのインデックスを更新
      const activeTabId = getSelectedTab()?.id;
      if (activeTabId !== undefined) {
        const newActiveIndex = reorderedTabs.findIndex(tab => tab.id === activeTabId);
        if (newActiveIndex !== -1) {
          selected.set(newActiveIndex);
        } else {
          // 万が一アクティブタブが見つからなくなったら先頭を選択（あるいはエラー処理）
          selected.set(0);
        }
      } else if (reorderedTabs.length > 0) {
        selected.set(0); // アクティブタブが不明でタブが存在する場合は先頭を選択
      } else {
        selected.set(-1); // タブがなくなったら-1
      }
      return reorderedTabs;
    });
  };
  // --- ここまで追加 ---

  return {
    tabs, // getTabsではなく、writableなtabsストア自体を返す
    selected, // getSelectedではなく、writableなselectedストア自体を返す
    getSelectedTab,
    routeLoaded,
    deleteTab,
    initialize,
    reorderTabs, // 追加
  };
};

// createTabs() の戻り値の型を明示的に定義 (省略可能だが可読性のため)
interface TabsStore {
  tabs: ReturnType<typeof createLocalStorageWritable<Tab[]>>[0];
  selected: ReturnType<typeof createLocalStorageWritable<number>>[0];
  getSelectedTab: () => Tab | undefined;
  routeLoaded: (event: RouteLoadedEvent) => void;
  deleteTab: (id: number) => void;
  initialize: () => void;
  reorderTabs: (oldIndex: number, newIndex: number) => void;
}

const createdTabs: TabsStore = createTabs();

export const {
  tabs,
  selected,
  getSelectedTab,
  routeLoaded,
  deleteTab,
  initialize,
  reorderTabs, // 追加
} = createdTabs;
