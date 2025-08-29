import { createLocalStorageWritable, localStorageWritable } from "@/lib/utils";
import { push, type RouteLoadedEvent } from "svelte-spa-router";

export type Tab = {
  id: number;
  workId?: number;
  type: "works" | "memos" | "settings";
  scrollTo: number;
  title: string;
  path: string; // ★ path を必須プロパティに変更 (設定タブも明確なパスを持つため)
};

const isValidTabType = (src: string): src is "works" | "memos" | "settings" => {
  return src === "works" || src === "memos" || src === "settings";
};

// 設定タブは「プレイ状況一括設定」専用とする
const PLAY_STATUS_EDITOR_TAB_ID = -100; // 固定IDは維持しても良いが、重複しないように注意
const PLAY_STATUS_EDITOR_PATH = "/settings/play-status";
const PLAY_STATUS_EDITOR_TITLE = "プレイ状況一括編集";

const SHORTCUT_SETTINGS_TAB_ID = -101;
const SHORTCUT_SETTINGS_PATH = "/settings/shortcut";
const SHORTCUT_SETTINGS_TITLE = "ショートカット設定";

const createTabs = () => {
  const [tabs, getTabs] = createLocalStorageWritable<Tab[]>("tabs", []);
  const [selected, getSelected] = createLocalStorageWritable("tab-selected", -1);

  const routeLoaded = (event: RouteLoadedEvent) => {
    const location = event.detail.location;
    localStorage.setItem("last-path", location);
    const params = event.detail.params;

    if (location === "/") {
      selected.set(-1);
      return;
    }

    const pathSegments = location.split("/").filter(Boolean);
    const tabTypeSegment = pathSegments[0];
    let entityId: number | undefined = undefined;
    if (pathSegments[1] && (tabTypeSegment === "works" || tabTypeSegment === "memos")) {
        entityId = +pathSegments[1];
    }

    if (!isValidTabType(tabTypeSegment)) {
      console.error("Invalid tab type from route:", tabTypeSegment);
      push("/");
      return;
    }

    let tabToSelectIndex = -1;

    if (location === PLAY_STATUS_EDITOR_PATH && tabTypeSegment === "settings") { // ★ 設定タブのパスで判定
        tabToSelectIndex = getTabs().findIndex(t => t.id === PLAY_STATUS_EDITOR_TAB_ID && t.type === "settings");
        if (tabToSelectIndex === -1) {
            const settingsTab: Tab = {
                id: PLAY_STATUS_EDITOR_TAB_ID,
                type: "settings",
                scrollTo: 0,
                title: PLAY_STATUS_EDITOR_TITLE,
                path: PLAY_STATUS_EDITOR_PATH,
            };
            tabs.update(currentTabs => [...currentTabs, settingsTab]);
            tabToSelectIndex = getTabs().length - 1;
        }
    } else if (location === SHORTCUT_SETTINGS_PATH && tabTypeSegment === "settings") {
        tabToSelectIndex = getTabs().findIndex(t => t.id === SHORTCUT_SETTINGS_TAB_ID && t.type === "settings");
        if (tabToSelectIndex === -1) {
            const settingsTab: Tab = {
                id: SHORTCUT_SETTINGS_TAB_ID,
                type: "settings",
                scrollTo: 0,
                title: SHORTCUT_SETTINGS_TITLE,
                path: SHORTCUT_SETTINGS_PATH,
            };
            tabs.update(currentTabs => [...currentTabs, settingsTab]);
            tabToSelectIndex = getTabs().length - 1;
        }
    } else if (tabTypeSegment === "works" || tabTypeSegment === "memos") {
        if (!entityId || isNaN(entityId)) {
            console.error("Missing or invalid entityId for works/memos tab at location:", location);
            // entityId がない場合は、新しいタブを開くのではなく、既存のタブを探すだけにするか、
            // あるいはエラーとしてホームに戻す。ここでは既存のタブを探す試みは維持。
            // もし対応する workId のタブがなければ、何もしないか、ホームに戻る。
            const existingTabIndex = getTabs().findIndex(t => t.workId === entityId && t.type === tabTypeSegment);
            if (existingTabIndex !== -1) {
                tabToSelectIndex = existingTabIndex;
            } else {
                // gamename がないとタイトルが作れないので、新しいタブは開かない
                const searchParams = new URLSearchParams(event.detail.querystring);
                const gamename = searchParams.get("gamename");
                if (gamename) {
                    let title = gamename;
                    if (tabTypeSegment === "memos") {
                        title = `メモ - ${title}`;
                    }
                    const newTab: Tab = {
                        id: new Date().getTime(),
                        workId: entityId,
                        type: tabTypeSegment,
                        scrollTo: 0,
                        title,
                        path: `/${tabTypeSegment}/${entityId}${event.detail.querystring ? `?${event.detail.querystring}` : ''}`,
                    };
                    tabs.update(v => [...v, newTab]);
                    tabToSelectIndex = getTabs().length - 1;
                } else {
                    console.warn(`Cannot open new ${tabTypeSegment} tab for ${entityId} without gamename.`);
                    // push("/"); // 必要ならホームに戻す
                }
            }
        } else {
            tabToSelectIndex = getTabs().findIndex(t => t.workId === entityId && t.type === tabTypeSegment);
            if (tabToSelectIndex === -1) {
                const searchParams = new URLSearchParams(event.detail.querystring);
                const gamename = searchParams.get("gamename");
                if (!gamename) {
                    console.error(`Gamename query param missing for new ${tabTypeSegment} tab for workId ${entityId}`);
                }
                let title = gamename || `ID: ${entityId}`;
                if (tabTypeSegment === "memos") {
                    title = `メモ - ${title}`;
                }
                const newTab: Tab = {
                    id: new Date().getTime(),
                    workId: entityId,
                    type: tabTypeSegment,
                    scrollTo: 0,
                    title,
                    path: `/${tabTypeSegment}/${entityId}${event.detail.querystring ? `?${event.detail.querystring}` : ''}`,
                };
                tabs.update(v => [...v, newTab]);
                tabToSelectIndex = getTabs().length - 1;
            }
        }
    }


    if (tabToSelectIndex !== -1) {
        selected.set(tabToSelectIndex);
    } else if (location !== "/" && location !== PLAY_STATUS_EDITOR_PATH) { // ★設定タブ以外で適切なタブが見つからなければ
        // 既に適切なURLにいるがタブリストにない場合（例：URL直打ちでquerystringなし）
        // この状態を許容するか、ホームに戻すか。
        // console.warn("No tab found for current route, staying or pushing home:", location);
        // push("/");
    }
  };

  const deleteTab = (idToDelete: number) => {
    const currentTabs = getTabs();
    const deleteIndex = currentTabs.findIndex(t => t.id === idToDelete);
    if (deleteIndex === -1) return;

    const currentSelectedRaw = getSelected();
    let newSelectedRaw = currentSelectedRaw;

    const newTabs = currentTabs.filter(t => t.id !== idToDelete);
    tabs.set(newTabs);

    if (newTabs.length === 0) {
      selected.set(-1);
      push("/");
      return;
    }

    if (currentSelectedRaw === deleteIndex) {
      newSelectedRaw = Math.max(0, deleteIndex - 1);
      const nextTabToPush = newTabs[newSelectedRaw];
      if (nextTabToPush) {
        push(nextTabToPush.path); // path を使う
      } else {
        push("/");
      }
    } else if (currentSelectedRaw > deleteIndex) {
      newSelectedRaw = currentSelectedRaw - 1;
    }
    selected.set(newSelectedRaw);
  };


  const initialize = () => {
    const lastPath = localStorage.getItem("last-path");
    // If we have a last path, and the current path is the root
    // (which is the default when reopening the app), then navigate.
    if (lastPath && window.location.pathname === "/") {
      push(lastPath);
      return; // The push will trigger routeLoaded, which will handle the rest.
    }

    // For other cases (like a page reload), we don't need to push a URL.
    // The browser is already at the correct URL.
    // We just need to ensure our tab state is consistent.
    const _tabs = getTabs();
    const index = getSelected();

    if (_tabs.length === 0) {
      selected.set(-1);
      return;
    }
    if (index < 0 || index >= _tabs.length) {
      // The selected tab index is invalid.
      // We can't know which tab should be active, so we set to none.
      // The `routeLoaded` event will soon fire for the current URL
      // and select the correct tab anyway.
      selected.set(-1);
    }
  };

  const getSelectedTab = (): Tab | undefined => {
    const selIndex = getSelected();
    const currentTabs = getTabs();
    if (selIndex >= 0 && selIndex < currentTabs.length) {
        return currentTabs[selIndex];
    }
    return undefined;
  };

  const reorderTabs = (oldIndex: number, newIndex: number) => {
    tabs.update(currentTabs => {
      if (oldIndex < 0 || oldIndex >= currentTabs.length || newIndex < 0 || newIndex > currentTabs.length) {
        return currentTabs;
      }
      const itemToMove = currentTabs[oldIndex];
      const remainingItems = currentTabs.filter((_, index) => index !== oldIndex);
      const reorderedTabs = [
        ...remainingItems.slice(0, newIndex),
        itemToMove,
        ...remainingItems.slice(newIndex)
      ];
      const activeTabId = getSelectedTab()?.id;
      if (activeTabId !== undefined) {
        const newActiveIndex = reorderedTabs.findIndex(tab => tab.id === activeTabId);
        selected.set(newActiveIndex !== -1 ? newActiveIndex : 0);
      } else if (reorderedTabs.length > 0) {
        selected.set(0);
      } else {
        selected.set(-1);
      }
      return reorderedTabs;
    });
  };

  const openSettingsTab = () => {
    const currentOpenTabs = getTabs();
    const settingsTabIndex = currentOpenTabs.findIndex(t => t.id === PLAY_STATUS_EDITOR_TAB_ID);

    if (settingsTabIndex !== -1) {
      selected.set(settingsTabIndex);
      push(PLAY_STATUS_EDITOR_PATH);
    } else {
      const settingsTab: Tab = {
        id: PLAY_STATUS_EDITOR_TAB_ID,
        type: "settings",
        scrollTo: 0,
        title: PLAY_STATUS_EDITOR_TITLE,
        path: PLAY_STATUS_EDITOR_PATH,
      };
      tabs.update(v => [...v, settingsTab]);
      selected.set(getTabs().length - 1);
      push(PLAY_STATUS_EDITOR_PATH);
    }
  };

  const openShortcutSettingsTab = () => {
    const currentOpenTabs = getTabs();
    const settingsTabIndex = currentOpenTabs.findIndex(t => t.id === SHORTCUT_SETTINGS_TAB_ID);

    if (settingsTabIndex !== -1) {
      selected.set(settingsTabIndex);
      push(SHORTCUT_SETTINGS_PATH);
    } else {
      const settingsTab: Tab = {
        id: SHORTCUT_SETTINGS_TAB_ID,
        type: "settings",
        scrollTo: 0,
        title: SHORTCUT_SETTINGS_TITLE,
        path: SHORTCUT_SETTINGS_PATH,
      };
      tabs.update(v => [...v, settingsTab]);
      selected.set(getTabs().length - 1);
      push(SHORTCUT_SETTINGS_PATH);
    }
  };

  return {
    tabs,
    selected,
    getSelectedTab,
    routeLoaded,
    deleteTab,
    initialize,
    reorderTabs,
    openSettingsTab,
    openShortcutSettingsTab,
  };
};

interface TabsStore {
  tabs: ReturnType<typeof createLocalStorageWritable<Tab[]>>[0];
  selected: ReturnType<typeof createLocalStorageWritable<number>>[0];
  getSelectedTab: () => Tab | undefined;
  routeLoaded: (event: RouteLoadedEvent) => void;
  deleteTab: (idToDelete: number) => void;
  initialize: () => void;
  reorderTabs: (oldIndex: number, newIndex: number) => void;
  openSettingsTab: () => void;
  openShortcutSettingsTab: () => void;
}

const createdTabs: TabsStore = createTabs();

export const {
  tabs,
  selected,
  getSelectedTab,
  routeLoaded,
  deleteTab,
  initialize,
  reorderTabs,
  openSettingsTab,
  openShortcutSettingsTab,
} = createdTabs;
