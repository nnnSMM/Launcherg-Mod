import type {
  AllGameCacheOne,
  CollectionElement,
  CollectionElementDetail,
  PlayStatus,
  Work,
} from "@/lib/types";
import demoData from "@/mock/demoData.json";
import {
  allGameCaches,
  getDetailSeedById,
  getGameCacheById,
  getWorkById,
} from "@/mock/demoCatalog";
import {
  getBestGamePathMatches,
  getGameCandidatesByFilePath,
  normalizeForGameMatch,
} from "@/mock/demoGameMatching";
import { scanDemoPaths } from "@/mock/demoBrowserFiles";
import { getDemoIconUrlForPath } from "@/mock/demoIconExtraction";
import { emit } from "@/mock/tauri-event";

const STORAGE_KEY = "launcherg-demo-collection-elements-v6";
const SETTINGS_KEY = "launcherg-demo-settings-v1";

const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value));

const initialCollectionElements = () =>
  clone(demoData.collectionElements as unknown as CollectionElement[]);

const loadCollectionElements = () => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return JSON.parse(stored) as CollectionElement[];
    }
  } catch (e) {
    console.warn("[Mock Tauri Core] failed to load collection elements", e);
  }
  return initialCollectionElements();
};

let collectionElements = loadCollectionElements();
let pauseState = false;

const saveCollectionElements = () => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(collectionElements));
  } catch (e) {
    console.warn("[Mock Tauri Core] failed to save collection elements", e);
  }
};

const loadSettings = () => {
  try {
    return JSON.parse(localStorage.getItem(SETTINGS_KEY) ?? "{}") as Record<
      string,
      string | null
    >;
  } catch {
    return {};
  }
};

const saveSettings = (settings: Record<string, string | null>) => {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
};

const nowString = () => new Date().toISOString().slice(0, 19).replace("T", " ");

const isLnkLike = (path: string | null | undefined) => {
  const lower = (path ?? "").toLowerCase();
  return lower.endsWith(".lnk") || lower.endsWith(".url");
};

const parentFolderKey = (path: string) =>
  path
    .split(/[\\/]/)
    .filter(Boolean)
    .slice(0, -1)
    .map(normalizeForGameMatch)
    .join("/");

const getImageSize = async (src: string) => {
  if (!src || typeof Image === "undefined" || import.meta.env.MODE === "test") {
    return null;
  }

  return await new Promise<{ width: number; height: number } | null>((resolve) => {
    const img = new Image();
    const timeout = window.setTimeout(() => resolve(null), 1500);
    img.onload = () => {
      window.clearTimeout(timeout);
      resolve({ width: img.naturalWidth, height: img.naturalHeight });
    };
    img.onerror = () => {
      window.clearTimeout(timeout);
      resolve(null);
    };
    img.src = src;
  });
};

const createCollectionElement = async (
  gameCache: AllGameCacheOne,
  path: { exePath: string | null; lnkPath: string | null },
  extractedIcon: string | null = null,
): Promise<CollectionElement> => {
  const seed = getDetailSeedById(gameCache.id);
  const previous = collectionElements.find((element) => element.id === gameCache.id);
  const timestamp = nowString();
  const thumbnail = previous?.thumbnail ?? gameCache.thumbnailUrl;
  const imageSize =
    previous?.thumbnailWidth && previous?.thumbnailHeight
      ? null
      : await getImageSize(thumbnail);
  const previousIcon =
    previous?.icon && previous.icon !== "images/dummy_thumbnail.svg"
      ? previous.icon
      : null;
  const imageIcon =
    extractedIcon ?? previousIcon ?? (thumbnail || "images/dummy_thumbnail.svg");

  return {
    id: gameCache.id,
    gamename: gameCache.gamename,
    gamenameRuby: previous?.gamenameRuby ?? seed.gamenameRuby,
    brandname: previous?.brandname ?? seed.brandname,
    brandnameRuby: previous?.brandnameRuby ?? seed.brandnameRuby,
    sellday: previous?.sellday ?? seed.sellday,
    isNukige: previous?.isNukige ?? seed.isNukige,
    installAt: previous?.installAt ?? timestamp,
    firstPlayAt: previous?.firstPlayAt ?? null,
    lastPlayAt: previous?.lastPlayAt ?? null,
    likeAt: previous?.likeAt ?? null,
    playStatus: previous?.playStatus ?? 0,
    totalPlayTimeSeconds: previous?.totalPlayTimeSeconds ?? 0,
    registeredAt: previous?.registeredAt ?? timestamp,
    exePath: path.exePath ?? previous?.exePath ?? "",
    lnkPath: path.lnkPath ?? previous?.lnkPath ?? "",
    icon: imageIcon,
    thumbnail,
    thumbnailWidth: previous?.thumbnailWidth ?? imageSize?.width ?? null,
    thumbnailHeight: previous?.thumbnailHeight ?? imageSize?.height ?? null,
    updatedAt: timestamp,
  };
};

const upsertElement = async (
  gameCache: AllGameCacheOne,
  path: { exePath: string | null; lnkPath: string | null },
  extractedIcon: string | null = null,
) => {
  const element = await createCollectionElement(gameCache, path, extractedIcon);
  const index = collectionElements.findIndex((v) => v.id === element.id);
  if (index >= 0) {
    collectionElements[index] = element;
  } else {
    collectionElements = [element, ...collectionElements];
  }
  saveCollectionElements();
  return element;
};

const getDemoIconUrlForElementPath = async (path: {
  exePath: string | null;
  lnkPath: string | null;
}) => {
  const elementPath = path.exePath || path.lnkPath;
  return elementPath ? await getDemoIconUrlForPath(elementPath) : null;
};

const getAllWorks = (): Work[] =>
  collectionElements
    .map((element) => getWorkById(element.id))
    .filter((work): work is Work => Boolean(work));

const searchAllGameCache = (query: string, limit: number, offset: number) => {
  const normalizedQuery = normalizeForGameMatch(query);
  const source = normalizedQuery
    ? allGameCaches.filter((cache) =>
        normalizeForGameMatch(cache.gamename).includes(normalizedQuery),
      )
    : allGameCaches;
  return source.slice(offset, offset + limit);
};

const seiyaUrlByNamePart: [string, string][] = [
  ["紙の上の魔法使い", "https://seiya-saiga.com/game/uguisukagura/kamimaho.html"],
  ["創作彼女", "https://seiya-saiga.com/game/ainolinks/soukano.html"],
  ["ディメンション", "https://seiya-saiga.com/game/crystalia/dimensionlovers.html"],
];

export const invoke = async <T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> => {
  console.log("[Mock Tauri Core] invoke:", cmd, args);

  if (cmd === "get_all_elements") {
    return collectionElements as T;
  }

  if (cmd === "get_all_works") {
    return getAllWorks() as T;
  }

  if (cmd === "get_collection_element") {
    const id = Number(args?.collectionElementId);
    return (
      collectionElements.find((game) => Number(game.id) === id) ??
      collectionElements[0]
    ) as T;
  }

  if (cmd === "get_nearest_key_and_distance") {
    const name = String(args?.key ?? "");
    const result = seiyaUrlByNamePart.find(([namePart]) =>
      name.includes(namePart),
    );
    return (result ? [result[1], 0] : ["", 100]) as T;
  }

  if (cmd === "get_default_import_dirs") {
    return [] as T;
  }

  if (cmd === "get_game_candidates") {
    const filepath = String(args?.filepath ?? "");
    return getGameCandidatesByFilePath(filepath, 0.2, 5).map((cache) => [
      cache.id,
      cache.gamename,
    ]) as T;
  }

  if (cmd === "create_elements_in_pc") {
    const paths = (args?.exploreDirPaths ?? []) as string[];
    const files = await scanDemoPaths(paths);
    await emit("progresslive", { max: files.length });

    const additions = await getBestGamePathMatches(
      files.map((file) => file.path),
      async () => {
        await emit("progresslive", {});
      },
    );
    for (const id of Array.from(additions.keys())) {
      if (collectionElements.some((element) => element.id === id)) {
        additions.delete(id);
      }
    }

    const addedNames: string[] = [];
    for (const { cache, path } of additions.values()) {
      const extractedIcon = await getDemoIconUrlForPath(path);
      await upsertElement(cache, {
        exePath: isLnkLike(path) ? null : path,
        lnkPath: isLnkLike(path) ? path : null,
      }, extractedIcon);
      addedNames.push(cache.gamename);
    }

    return addedNames as T;
  }

  if (cmd === "preview_demo_game_matching") {
    const paths = (args?.exploreDirPaths ?? []) as string[];
    const files = await scanDemoPaths(paths);
    await emit("progresslive", { max: files.length });
    const matchedByGame = await getBestGamePathMatches(
      files.map((file) => file.path),
      async () => {
        await emit("progresslive", {});
      },
    );
    const matchedPathSet = new Set(
      Array.from(matchedByGame.values()).map((match) => match.path),
    );
    const highConfidenceByPath = new Map<string, AllGameCacheOne | null>();
    const getHighConfidenceCandidate = (path: string) => {
      if (!highConfidenceByPath.has(path)) {
        highConfidenceByPath.set(
          path,
          getGameCandidatesByFilePath(path, 0.8, 1)[0] ?? null,
        );
      }
      return highConfidenceByPath.get(path) ?? null;
    };
    const highConfidenceFolderSet = new Set(
      files
        .filter((file) => getHighConfidenceCandidate(file.path))
        .map((file) => parentFolderKey(file.path)),
    );

    const matchedResults = Array.from(matchedByGame.values()).map(({ cache, path }) => ({
      path,
      matched: {
        id: cache.id,
        gamename: cache.gamename,
        thumbnailUrl: cache.thumbnailUrl,
      },
      candidates: getGameCandidatesByFilePath(path, 0.2, 3).map((candidate) => [
        candidate.id,
        candidate.gamename,
      ]),
    }));
    const unmatchedResults = files
      .filter((file) => !matchedPathSet.has(file.path))
      .filter((file) => !highConfidenceFolderSet.has(parentFolderKey(file.path)))
      .map((file) => ({
        path: file.path,
        matched: null,
        candidates: getGameCandidatesByFilePath(file.path, 0.2, 3).map((cache) => [
          cache.id,
          cache.gamename,
        ]),
      }));

    return {
      scannedFileCount: files.length,
      matchedCount: matchedResults.length,
      results: [...matchedResults, ...unmatchedResults],
    } as T;
  }

  if (cmd === "upsert_collection_element") {
    const gameCache = args?.gameCache as AllGameCacheOne | undefined;
    if (gameCache) {
      const path = {
        exePath: (args?.exePath as string | null | undefined) ?? null,
        lnkPath: (args?.lnkPath as string | null | undefined) ?? null,
      };
      await upsertElement(gameCache, path, await getDemoIconUrlForElementPath(path));
    }
    return undefined as T;
  }

  if (cmd === "delete_collection_element" || cmd === "delete_collection_element_logical") {
    const id = Number(args?.collectionElementId ?? args?.id);
    collectionElements = collectionElements.filter((element) => element.id !== id);
    saveCollectionElements();
    return undefined as T;
  }

  if (cmd === "update_element_like") {
    const id = Number(args?.id);
    collectionElements = collectionElements.map((element) =>
      element.id === id
        ? { ...element, likeAt: args?.isLike ? nowString() : null }
        : element,
    );
    saveCollectionElements();
    return undefined as T;
  }

  if (cmd === "update_element_play_status") {
    const id = Number(args?.id);
    collectionElements = collectionElements.map((element) =>
      element.id === id
        ? { ...element, playStatus: args?.playStatus as PlayStatus }
        : element,
    );
    saveCollectionElements();
    return undefined as T;
  }

  if (cmd === "update_collection_element_path") {
    const id = Number(args?.id);
    const path = String(args?.path ?? "");
    const icon = await getDemoIconUrlForPath(path);
    collectionElements = collectionElements.map((element) =>
      element.id === id
        ? {
            ...element,
            exePath: isLnkLike(path) ? "" : path,
            lnkPath: isLnkLike(path) ? path : "",
            icon: icon ?? element.icon,
            updatedAt: nowString(),
          }
        : element,
    );
    saveCollectionElements();
    return undefined as T;
  }

  if (cmd === "update_game_image") {
    const id = Number(args?.elementId);
    const imageType = args?.imageType;
    const newImagePath = String(args?.newImagePath ?? "");
    collectionElements = collectionElements.map((element) =>
      element.id === id
        ? {
            ...element,
            icon: imageType === "icon" ? newImagePath : element.icon,
            thumbnail: imageType === "thumbnail" ? newImagePath : element.thumbnail,
            updatedAt: nowString(),
          }
        : element,
    );
    saveCollectionElements();
    return undefined as T;
  }

  if (cmd === "update_collection_element_icon") {
    const id = Number(args?.id);
    const path = String(args?.path ?? "");
    collectionElements = collectionElements.map((element) =>
      element.id === id ? { ...element, icon: path, updatedAt: nowString() } : element,
    );
    saveCollectionElements();
    return undefined as T;
  }

  if (cmd === "create_element_details") {
    const details = (args?.details ?? []) as CollectionElementDetail[];
    collectionElements = collectionElements.map((element) => {
      const detail = details.find((v) => v.collectionElementId === element.id);
      return detail
        ? {
            ...element,
            gamenameRuby: detail.gamenameRuby,
            brandname: detail.brandname,
            brandnameRuby: detail.brandnameRuby,
            sellday: detail.sellday,
            isNukige: detail.isNukige,
            updatedAt: nowString(),
          }
        : element;
    });
    saveCollectionElements();
    return undefined as T;
  }

  if (cmd === "get_not_registered_detail_element_ids") {
    return collectionElements
      .filter((element) => !element.brandname && !element.sellday)
      .map((element) => element.id) as T;
  }

  if (cmd === "search_all_game_cache") {
    return searchAllGameCache(
      String(args?.query ?? ""),
      Number(args?.limit ?? 20),
      Number(args?.offset ?? 0),
    ) as T;
  }

  if (cmd === "get_game_cache_all") {
    return allGameCaches as T;
  }

  if (cmd === "get_all_game_cache_last_updated") {
    const maxId = allGameCaches.reduce((max, cache) => Math.max(max, cache.id), 0);
    return [maxId, "2026-05-17T00:00:00Z"] as T;
  }

  if (cmd === "get_game_cache_by_id") {
    const id = Number(args?.id);
    return getGameCacheById(id) as T;
  }

  if (cmd === "get_exe_path_by_lnk") {
    return String(args?.filepath ?? "") as T;
  }

  if (cmd === "play_game") {
    const id = Number(args?.elementId);
    const playedAt = nowString();
    collectionElements = collectionElements.map((element) =>
      element.id === id
        ? {
            ...element,
            firstPlayAt: element.firstPlayAt ?? playedAt,
            lastPlayAt: playedAt,
            totalPlayTimeSeconds: element.totalPlayTimeSeconds + 60,
          }
        : element,
    );
    saveCollectionElements();
    return Math.floor(Math.random() * 100000) as T;
  }

  if (cmd === "get_play_time_minutes") {
    const id = Number(args?.collectionElementId);
    const element = collectionElements.find((v) => v.id === id);
    return Math.floor((element?.totalPlayTimeSeconds ?? 0) / 60) as T;
  }

  if (cmd === "get_app_setting") {
    const settings = loadSettings();
    if (args?.key === "shortcut_game_id" && settings.shortcut_game_id === undefined) {
      return "39837" as T;
    }
    return (settings[String(args?.key ?? "")] ?? null) as T;
  }

  if (cmd === "set_app_setting") {
    const settings = loadSettings();
    settings[String(args?.key ?? "")] = (args?.value as string | null | undefined) ?? null;
    saveSettings(settings);
    return undefined as T;
  }

  if (cmd === "toggle_pause_tracking") {
    pauseState = !pauseState;
    return pauseState as T;
  }

  if (cmd === "get_pause_state") return pauseState as T;
  if (cmd === "get_game_screenshots") return [] as T;
  if (cmd === "get_all_screenshots") return [] as T;
  if (cmd === "get_vndb_screenshot_cache") return null as T;

  if (
    cmd === "open_folder" ||
    cmd === "upload_image" ||
    cmd === "update_all_game_cache" ||
    cmd === "save_screenshot_by_pid" ||
    cmd === "import_screenshot" ||
    cmd === "delete_screenshot" ||
    cmd === "update_screenshots_order" ||
    cmd === "upsert_vndb_screenshot_cache" ||
    cmd === "open_screenshot_window" ||
    cmd === "launch_shortcut_game" ||
    cmd === "show_main_window" ||
    cmd === "hide_tray_menu" ||
    cmd === "save_main_window_state" ||
    cmd === "quit_app"
  ) {
    return null as T;
  }

  return null as T;
};

export const convertFileSrc = (p: string) => {
  if (p && (p.startsWith("http") || p.startsWith("blob:") || p.startsWith("data:"))) {
    return p;
  }
  if (p && (p.startsWith("demo-images/") || p.startsWith("images/"))) {
    return p;
  }
  return "images/dummy_thumbnail.svg";
};
