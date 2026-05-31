import { getVersion } from "@tauri-apps/api/app";
import { fetch } from "@tauri-apps/plugin-http";
import { open } from "@tauri-apps/plugin-shell";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  check,
  type DownloadEvent,
  type Update as TauriUpdate,
} from "@tauri-apps/plugin-updater";
import { writable } from "svelte/store";
import { commandGetAppSetting, commandSetAppSetting } from "@/lib/command";

const IGNORED_UPDATE_VERSION_KEY = "ignored_update_version";
const MOCK_UPDATE_STORAGE_KEY = "launcherg-update-mock";
const updateInfoUrl =
  "https://github.com/nnnSMM/Launcherg-Mod/releases/latest/download/update-info.json";

export const demoUrl = "https://nnnsmm.github.io/Launcherg-Mod/";
export const releaseUrl = "https://github.com/nnnSMM/Launcherg-Mod/releases";

type UpdateStatus =
  | "idle"
  | "checking"
  | "none"
  | "available"
  | "ignored"
  | "installing"
  | "installed"
  | "error";

export type AppUpdateInfo = {
  currentVersion: string;
  currentDisplayVersion: string;
  version: string;
  displayVersion: string;
  date: string | null;
  body: string;
  demoUrl: string;
  releaseUrl: string;
  highlights: string[];
  isMock: boolean;
  canInstall: boolean;
};

type UpdateInfoJson = {
  version?: string;
  displayVersion?: string;
  demoUrl?: string;
  releaseUrl?: string;
  highlights?: string[];
};

type AppUpdateState = {
  status: UpdateStatus;
  update: AppUpdateInfo | null;
  isDialogOpen: boolean;
  installMessage: string;
  installProgress: number | null;
  error: string | null;
};

const initialState: AppUpdateState = {
  status: "idle",
  update: null,
  isDialogOpen: false,
  installMessage: "",
  installProgress: null,
  error: null,
};

const displayVersion = (version: string) => {
  const dateVersion = version.match(/^(\d{2})\.(\d{1,2})\.(\d{1,2})$/);
  if (!dateVersion) {
    return version;
  }
  const [, year, month, day] = dateVersion;
  return `20${year}${month.padStart(2, "0")}${day.padStart(2, "0")}`;
};

const releaseUrlForVersion = (version: string) =>
  `${releaseUrl}/tag/${encodeURIComponent(displayVersion(version))}`;

const errorMessage = (error: unknown) =>
  error instanceof Error ? error.message : String(error);

const getSearchParams = () => {
  const params = new URLSearchParams(window.location.search);
  const hashQueryIndex = window.location.hash.indexOf("?");
  if (hashQueryIndex >= 0) {
    const hashParams = new URLSearchParams(
      window.location.hash.slice(hashQueryIndex + 1),
    );
    for (const [key, value] of hashParams.entries()) {
      params.set(key, value);
    }
  }
  return params;
};

const isTruthyMockFlag = (value: string | null) =>
  value === "1" || value === "true" || value === "available";

const shouldUseMockUpdate = () => {
  if (!import.meta.env.DEV && !__PUBLIC_DEMO_BUILD__) {
    return false;
  }

  const params = getSearchParams();
  if (isTruthyMockFlag(params.get("mockUpdate"))) {
    return true;
  }

  return isTruthyMockFlag(localStorage.getItem(MOCK_UPDATE_STORAGE_KEY));
};

const shouldResetIgnoredMockUpdate = () =>
  isTruthyMockFlag(getSearchParams().get("mockUpdate"));

const createMockUpdate = async (): Promise<AppUpdateInfo> => {
  let currentVersion = "0.3.3";
  try {
    currentVersion = await getVersion();
  } catch (e) {
    console.warn("Failed to get app version for mock update:", e);
  }

  return {
    currentVersion,
    currentDisplayVersion: displayVersion(currentVersion),
    version: "26.5.24",
    displayVersion: "20260524",
    date: new Date().toISOString(),
    body: [
      "更新通知UIの確認用モックです。",
      "demo ページへの導線、リリースノート導線、後で閉じる、バージョン無視を確認できます。",
      "このモックでは実際のダウンロードやインストールは実行しません。",
    ].join("\n"),
    demoUrl,
    releaseUrl: releaseUrlForVersion("20260524"),
    highlights: [
      "更新通知UIの表示",
      "demo ページとリリースノートへの導線",
      "明示操作後だけアップデートを実行する制御",
    ],
    isMock: true,
    canInstall: false,
  };
};

const loadUpdateInfoJson = async (version: string): Promise<UpdateInfoJson | null> => {
  try {
    const response = await fetch(updateInfoUrl, { method: "GET" });
    if (!response.ok) {
      return null;
    }
    const data = (await response.json()) as UpdateInfoJson;
    if (data.version && data.version !== version) {
      return null;
    }
    return data;
  } catch (e) {
    console.warn("Failed to load update UI metadata:", e);
    return null;
  }
};

const toUpdateInfo = async (tauriUpdate: TauriUpdate): Promise<AppUpdateInfo> => {
  const uiInfo = await loadUpdateInfoJson(tauriUpdate.version);
  return {
    currentVersion: tauriUpdate.currentVersion,
    currentDisplayVersion: displayVersion(tauriUpdate.currentVersion),
    version: tauriUpdate.version,
    displayVersion: uiInfo?.displayVersion ?? displayVersion(tauriUpdate.version),
    date: tauriUpdate.date ?? null,
    body: tauriUpdate.body ?? "",
    demoUrl: uiInfo?.demoUrl ?? demoUrl,
    releaseUrl: uiInfo?.releaseUrl ?? releaseUrlForVersion(tauriUpdate.version),
    highlights: uiInfo?.highlights ?? [],
    isMock: false,
    canInstall: true,
  };
};

const createAppUpdateStore = () => {
  const { subscribe, set, update } = writable<AppUpdateState>(initialState);
  let didCheck = false;
  let activeTauriUpdate: TauriUpdate | null = null;

  const applyAvailableUpdate = async (info: AppUpdateInfo, tauriUpdate: TauriUpdate | null) => {
    const ignoredVersion = await commandGetAppSetting(IGNORED_UPDATE_VERSION_KEY);
    if (ignoredVersion === info.version) {
      activeTauriUpdate = null;
      update((state) => ({
        ...state,
        status: "ignored",
        update: null,
        isDialogOpen: false,
        error: null,
      }));
      return;
    }

    activeTauriUpdate = tauriUpdate;
    update((state) => ({
      ...state,
      status: "available",
      update: info,
      error: null,
      installMessage: "",
      installProgress: null,
    }));
  };

  const initialize = async () => {
    if (didCheck) {
      return;
    }
    didCheck = true;

    update((state) => ({ ...state, status: "checking", error: null }));

    try {
      if (shouldUseMockUpdate()) {
        if (shouldResetIgnoredMockUpdate()) {
          await commandSetAppSetting(IGNORED_UPDATE_VERSION_KEY, null);
        }
        await applyAvailableUpdate(await createMockUpdate(), null);
        return;
      }

      if (import.meta.env.DEV || __PUBLIC_DEMO_BUILD__) {
        activeTauriUpdate = null;
        update((state) => ({ ...state, status: "none", update: null }));
        return;
      }

      const checkedUpdate = await check({ timeout: 10000 });
      if (!checkedUpdate) {
        activeTauriUpdate = null;
        update((state) => ({ ...state, status: "none", update: null }));
        return;
      }

      await applyAvailableUpdate(await toUpdateInfo(checkedUpdate), checkedUpdate);
    } catch (e) {
      console.warn("Failed to check updates:", e);
      activeTauriUpdate = null;
      update((state) => ({
        ...state,
        status: "error",
        update: null,
        error: errorMessage(e),
      }));
    }
  };

  const openDialog = () => {
    update((state) => ({ ...state, isDialogOpen: true }));
  };

  const toggleDialog = () => {
    update((state) => ({ ...state, isDialogOpen: !state.isDialogOpen }));
  };

  const closeDialog = () => {
    update((state) => ({ ...state, isDialogOpen: false, error: null }));
  };

  const ignoreCurrentVersion = async () => {
    let version: string | null = null;
    update((state) => {
      version = state.update?.version ?? null;
      return state;
    });
    if (!version) {
      return;
    }
    await commandSetAppSetting(IGNORED_UPDATE_VERSION_KEY, version);
    activeTauriUpdate = null;
    update((state) => ({
      ...state,
      status: "ignored",
      update: null,
      isDialogOpen: false,
      error: null,
    }));
  };

  const openDemoPage = async () => {
    let url = demoUrl;
    update((state) => {
      url = state.update?.demoUrl ?? demoUrl;
      return state;
    });
    await open(url);
  };

  const openReleaseNotes = async (url: string) => {
    await open(url);
  };

  const installUpdate = async () => {
    if (!activeTauriUpdate) {
      update((state) => ({
        ...state,
        error: "demo または mock ではアップデートを実行できません。",
      }));
      return;
    }

    let downloaded = 0;
    let contentLength = 0;
    const onDownloadEvent = (event: DownloadEvent) => {
      if (event.event === "Started") {
        downloaded = 0;
        contentLength = event.data.contentLength ?? 0;
        update((state) => ({
          ...state,
          installMessage: "アップデートをダウンロードしています。",
          installProgress: contentLength ? 0 : null,
        }));
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        update((state) => ({
          ...state,
          installProgress: contentLength
            ? Math.min(downloaded / contentLength, 1)
            : null,
        }));
      } else if (event.event === "Finished") {
        update((state) => ({
          ...state,
          installMessage: "インストールを完了しています。",
          installProgress: 1,
        }));
      }
    };

    update((state) => ({
      ...state,
      status: "installing",
      error: null,
      installMessage: "アップデートを開始しています。",
      installProgress: null,
    }));

    try {
      await activeTauriUpdate.downloadAndInstall(onDownloadEvent);
      update((state) => ({
        ...state,
        status: "installed",
        installMessage: "アップデートが完了しました。再起動します。",
        installProgress: 1,
      }));
      await relaunch();
    } catch (e) {
      console.error("Failed to install update:", e);
      update((state) => ({
        ...state,
        status: "available",
        error: errorMessage(e),
        installMessage: "",
        installProgress: null,
      }));
    }
  };

  return {
    subscribe,
    initialize,
    openDialog,
    toggleDialog,
    closeDialog,
    ignoreCurrentVersion,
    openDemoPage,
    openReleaseNotes,
    installUpdate,
    resetForTest: () => {
      didCheck = false;
      activeTauriUpdate = null;
      set(initialState);
    },
  };
};

export const appUpdate = createAppUpdateStore();
