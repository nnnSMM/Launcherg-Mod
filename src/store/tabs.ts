import { createLocalStorageWritable } from "@/lib/utils";
import { push, replace, type RouteLoadedEvent } from "svelte-spa-router";

let isDemoBuildVal = import.meta.env.BASE_URL === "./";

export const isDemoBuild = () => isDemoBuildVal;

export const setDemoBuildForTest = (val: boolean) => {
  isDemoBuildVal = val;
};

export type Tab = {
  id: number;
  workId?: number;
  type: "works" | "memos" | "settings";
  scrollTo: number;
  title: string;
  path: string;
};

export const isValidTabType = (
  src: string,
): src is "works" | "memos" | "settings" => {
  return src === "works" || src === "memos" || src === "settings";
};

const PLAY_STATUS_EDITOR_TAB_ID = -100;
const PLAY_STATUS_EDITOR_PATH = "/settings/play-status";
const PLAY_STATUS_EDITOR_TITLE = "プレイ状態一括編集";

const SHORTCUT_SETTINGS_TAB_ID = -101;
const SHORTCUT_SETTINGS_PATH = "/settings/shortcut";
const SHORTCUT_SETTINGS_TITLE = "ショートカット設定";

const LEGACY_DISPLAY_SETTINGS_PATH = "/settings/display";
const ROOT_PATHS = new Set(["/", "/demo"]);

const createTabs = () => {
  const [tabs, getTabs] = createLocalStorageWritable<Tab[]>("tabs", []);
  const [selected, getSelected] = createLocalStorageWritable(
    "tab-selected",
    -1,
  );

  const insertNewTab = (newTab: Tab, preferredIndex?: number): number => {
    const currentIndex = getSelected();
    const currentTabs = getTabs();
    const insertIndex =
      preferredIndex !== undefined && preferredIndex >= 0
        ? preferredIndex
        : currentIndex === -1
        ? currentTabs.length
        : currentIndex + 1;

    tabs.update((current) => {
      const next = [...current];
      next.splice(insertIndex, 0, newTab);
      return next;
    });

    return insertIndex;
  };

  const findOrCreateSettingsTab = (
    tabId: number,
    tabPath: string,
    tabTitle: string,
  ): number => {
    let index = getTabs().findIndex(
      (tab) => tab.id === tabId && tab.type === "settings",
    );

    if (index === -1) {
      index = insertNewTab({
        id: tabId,
        type: "settings",
        scrollTo: 0,
        title: tabTitle,
        path: tabPath,
      });
    }

    return index;
  };

  const getSelectedIndexForLocation = (rawLocation: string): number => {
    const location =
      rawLocation === LEGACY_DISPLAY_SETTINGS_PATH
        ? SHORTCUT_SETTINGS_PATH
        : rawLocation;

    if (ROOT_PATHS.has(location)) {
      return -1;
    }

    if (location === PLAY_STATUS_EDITOR_PATH) {
      return getTabs().findIndex(
        (tab) =>
          tab.id === PLAY_STATUS_EDITOR_TAB_ID && tab.type === "settings",
      );
    }

    if (location === SHORTCUT_SETTINGS_PATH) {
      return getTabs().findIndex(
        (tab) =>
          tab.id === SHORTCUT_SETTINGS_TAB_ID && tab.type === "settings",
      );
    }

    const pathSegments = location.split("/").filter(Boolean);
    const tabTypeSegment = pathSegments[0];
    const entityId =
      pathSegments[1] &&
      (tabTypeSegment === "works" || tabTypeSegment === "memos")
        ? Number(pathSegments[1])
        : undefined;

    if (
      !isValidTabType(tabTypeSegment) ||
      tabTypeSegment === "settings" ||
      !entityId ||
      Number.isNaN(entityId)
    ) {
      return -1;
    }

    return getTabs().findIndex(
      (tab) => tab.workId === entityId && tab.type === tabTypeSegment,
    );
  };

  const syncSelectedToLocation = (rawLocation: string) => {
    const nextSelectedIndex = getSelectedIndexForLocation(rawLocation);
    if (nextSelectedIndex !== getSelected()) {
      selected.set(nextSelectedIndex);
    }
  };

  const routeLoaded = (event: RouteLoadedEvent) => {
    const rawLocation = event.detail.location;
    const location =
      rawLocation === LEGACY_DISPLAY_SETTINGS_PATH
        ? SHORTCUT_SETTINGS_PATH
        : rawLocation;

    localStorage.setItem("last-path", location);

    if (ROOT_PATHS.has(location)) {
      selected.set(-1);
      return;
    }

    const pathSegments = location.split("/").filter(Boolean);
    const tabTypeSegment = pathSegments[0];
    const entityId =
      pathSegments[1] && (tabTypeSegment === "works" || tabTypeSegment === "memos")
        ? Number(pathSegments[1])
        : undefined;

    if (!isValidTabType(tabTypeSegment)) {
      console.error("Invalid tab type from route:", tabTypeSegment);
      push(isDemoBuild() ? "/demo" : "/");
      return;
    }

    let tabToSelectIndex = -1;

    if (location === PLAY_STATUS_EDITOR_PATH && tabTypeSegment === "settings") {
      tabToSelectIndex = findOrCreateSettingsTab(
        PLAY_STATUS_EDITOR_TAB_ID,
        PLAY_STATUS_EDITOR_PATH,
        PLAY_STATUS_EDITOR_TITLE,
      );
    } else if (
      location === SHORTCUT_SETTINGS_PATH &&
      tabTypeSegment === "settings"
    ) {
      tabToSelectIndex = findOrCreateSettingsTab(
        SHORTCUT_SETTINGS_TAB_ID,
        SHORTCUT_SETTINGS_PATH,
        SHORTCUT_SETTINGS_TITLE,
      );
    } else if (tabTypeSegment === "works" || tabTypeSegment === "memos") {
      if (!entityId || Number.isNaN(entityId)) {
        console.error(
          "Missing or invalid entityId for works/memos tab at location:",
          location,
        );
        return;
      }

      tabToSelectIndex = getTabs().findIndex(
        (tab) => tab.workId === entityId && tab.type === tabTypeSegment,
      );

      if (tabToSelectIndex === -1) {
        const searchParams = new URLSearchParams(event.detail.querystring);
        const gamename = searchParams.get("gamename");
        let title = gamename || `ID: ${entityId}`;
        if (tabTypeSegment === "memos") {
          title = `メモ - ${title}`;
        }

        const correspondingWorkTabIndex = getTabs().findIndex(
          (tab) => tab.workId === entityId && tab.type === "works",
        );

        const preferredIndex =
          correspondingWorkTabIndex !== -1
            ? correspondingWorkTabIndex + 1
            : undefined;

        tabToSelectIndex = insertNewTab({
          id: new Date().getTime(),
          workId: entityId,
          type: tabTypeSegment,
          scrollTo: 0,
          title,
          path: `/${tabTypeSegment}/${entityId}${
            event.detail.querystring ? `?${event.detail.querystring}` : ""
          }`,
        }, preferredIndex);
      }
    }

    if (tabToSelectIndex !== -1) {
      selected.set(tabToSelectIndex);
    }
  };

  const deleteTab = (idToDelete: number) => {
    const currentTabs = getTabs();
    const deleteIndex = currentTabs.findIndex((tab) => tab.id === idToDelete);
    if (deleteIndex === -1) return;

    const currentSelectedRaw = getSelected();
    let newSelectedRaw = currentSelectedRaw;

    const newTabs = currentTabs.filter((tab) => tab.id !== idToDelete);
    tabs.set(newTabs);

    if (newTabs.length === 0) {
      selected.set(-1);
      replace(isDemoBuild() ? "/demo" : "/");
      return;
    }

    if (currentSelectedRaw === deleteIndex) {
      newSelectedRaw = Math.max(0, deleteIndex - 1);
      const nextTabToPush = newTabs[newSelectedRaw];
      if (nextTabToPush) {
        replace(nextTabToPush.path);
      } else {
        replace(isDemoBuild() ? "/demo" : "/");
      }
    } else if (currentSelectedRaw > deleteIndex) {
      newSelectedRaw = currentSelectedRaw - 1;
    }

    selected.set(newSelectedRaw);
  };

  const initialize = () => {
    tabs.update((currentTabs) =>
      currentTabs.filter((tab) => tab.path !== LEGACY_DISPLAY_SETTINGS_PATH),
    );

    const lastPath = localStorage.getItem("last-path");
    const normalizedLastPath =
      lastPath === LEGACY_DISPLAY_SETTINGS_PATH
        ? SHORTCUT_SETTINGS_PATH
        : lastPath;
    const currentHashPath = window.location.hash.replace(/^#/, "");
    const isRootHash = currentHashPath === "" || currentHashPath === "/";

    if (lastPath?.startsWith("/discover")) {
      localStorage.removeItem("last-path");
    }

    if (lastPath === LEGACY_DISPLAY_SETTINGS_PATH) {
      localStorage.setItem("last-path", SHORTCUT_SETTINGS_PATH);
    }

    if (
      normalizedLastPath &&
      window.location.pathname === "/" &&
      isRootHash &&
      !isDemoBuild()
    ) {
      push(normalizedLastPath);
      return;
    }

    const currentTabs = getTabs();
    const index = getSelected();

    if (currentTabs.length === 0) {
      selected.set(-1);
      return;
    }

    if (index < 0 || index >= currentTabs.length) {
      selected.set(-1);
    }
  };

  const getSelectedTab = (): Tab | undefined => {
    const selectedIndex = getSelected();
    const currentTabs = getTabs();
    if (selectedIndex >= 0 && selectedIndex < currentTabs.length) {
      return currentTabs[selectedIndex];
    }
    return undefined;
  };

  const reorderTabs = (oldIndex: number, newIndex: number) => {
    tabs.update((currentTabs) => {
      if (
        oldIndex < 0 ||
        oldIndex >= currentTabs.length ||
        newIndex < 0 ||
        newIndex > currentTabs.length
      ) {
        return currentTabs;
      }

      const itemToMove = currentTabs[oldIndex];
      const remainingItems = currentTabs.filter(
        (_, index) => index !== oldIndex,
      );
      const reorderedTabs = [
        ...remainingItems.slice(0, newIndex),
        itemToMove,
        ...remainingItems.slice(newIndex),
      ];
      const activeTabId = getSelectedTab()?.id;

      if (activeTabId !== undefined) {
        const newActiveIndex = reorderedTabs.findIndex(
          (tab) => tab.id === activeTabId,
        );
        selected.set(newActiveIndex !== -1 ? newActiveIndex : 0);
      } else if (reorderedTabs.length > 0) {
        selected.set(0);
      } else {
        selected.set(-1);
      }

      return reorderedTabs;
    });
  };

  const openSpecificSettingsTab = (
    tabId: number,
    tabPath: string,
    tabTitle: string,
  ) => {
    const existingTabIndex = getTabs().findIndex((tab) => tab.id === tabId);

    if (existingTabIndex !== -1) {
      selected.set(existingTabIndex);
      replace(tabPath);
      return;
    }

    const newIndex = insertNewTab({
      id: tabId,
      type: "settings",
      scrollTo: 0,
      title: tabTitle,
      path: tabPath,
    });
    selected.set(newIndex);
    push(tabPath);
  };

  const openSettingsTab = () => {
    openSpecificSettingsTab(
      PLAY_STATUS_EDITOR_TAB_ID,
      PLAY_STATUS_EDITOR_PATH,
      PLAY_STATUS_EDITOR_TITLE,
    );
  };

  const openShortcutSettingsTab = () => {
    openSpecificSettingsTab(
      SHORTCUT_SETTINGS_TAB_ID,
      SHORTCUT_SETTINGS_PATH,
      SHORTCUT_SETTINGS_TITLE,
    );
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
    syncSelectedToLocation,
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
  syncSelectedToLocation: (rawLocation: string) => void;
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
  syncSelectedToLocation,
} = createdTabs;
