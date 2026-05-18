import type { AllGameCacheOne } from "@/lib/types";
import { allGameCaches, getGameCacheById } from "@/mock/demoCatalog";

type NormalizedGameCache = AllGameCacheOne & {
  normalizedName: string;
};

export type DemoMatchedGamePath = {
  cache: AllGameCacheOne;
  path: string;
};

const STRICT_NOT_GAME_TERMS = [
  "\u30de\u30cb\u30e5\u30a2\u30eb",
  "\u8a73\u7d30\u8a2d\u5b9a",
  "\u306f\u3058\u3081\u306b",
  "\u30b5\u30dd\u30fc\u30c8",
  "\u30bb\u30fc\u30d6\u30c7\u30fc\u30bf",
  "\u30a4\u30f3\u30b9\u30c8\u30fc\u30eb",
  "\u30a2\u30f3\u30a4\u30f3\u30b9\u30c8\u30fc\u30eb",
  "\u4f53\u9a13\u7248",
  "install",
  "uninstall",
  "autorun",
  "\u524a\u9664",
  "license",
  "\u30e9\u30a4\u30bb\u30f3\u30b9",
  "\u516c\u5f0f\u30b5\u30a4\u30c8",
  "\u30db\u30fc\u30e0\u30da\u30fc\u30b8",
];

const IGNORE_WORD_WHEN_CONFLICT = [
  "\u8a2d\u5b9a",
  "\u30c1\u30a7\u30c3\u30af",
  "\u30a4\u30f3\u30b9\u30c8",
  "\u524a\u9664",
  "\u30d5\u30a1\u30a4\u30eb",
  "\u304f\u3060\u3055\u3044",
  "\u4e0b\u3055\u3044",
  "\u30de\u30cb\u30e5\u30a2\u30eb",
  "\u30a2\u30c3\u30d7\u30c7\u30fc\u30c8",
  "\u30b7\u30b9\u30c6\u30e0",
  "check",
  "setting",
  "config",
  "update",
  "inst",
  "tool",
  "support",
  "setup",
  "unins",
  "define",
  "bhvc",
  "bootstrap",
  "file",
  "exhibit",
  "ihs",
  "launcher",
  "syscfg",
  "updchk",
  "acmp",
];

const SHOULD_UPDATE_WORD_WHEN_CONFLICT = [
  "adv",
  "64",
  "cmvs",
  "bgi",
  "\u5b9f\u884c",
  "\u8d77\u52d5",
];

const ENGINE_NAMES = new Set(["bgi", "siglusengine", "nscripter"]);
const IGNORE_GAME_ID = new Set([2644, 63, 2797, 10419]);
const EQUAL_FILENAME_GAME_ID = new Map<string, number>([["pieces", 27123]]);
const PATH_SPECIFIC_GAME_IDS = [
  { grandparent: "枕", parent: "サクラノ詩", id: 4529 },
  { grandparent: "nekoneko", parent: "すみれ", id: 20178 },
];
const REMOVE_WORDS = [
  "\u3092\u8d77\u52d5",
  "\u306e\u8d77\u52d5",
  "_\u8d77\u52d5\u7528",
  "\u300c",
  "\u300d",
  " ",
  "\u3000",
  "\u30c0\u30a6\u30f3\u30ed\u30fc\u30c9\u7248",
  "dl\u7248",
];

export const normalizeForGameMatch = (value: string) => {
  let normalized = "";
  for (const char of value) {
    const code = char.codePointAt(0) ?? 0;
    if (code >= 0xff21 && code <= 0xff3a) {
      normalized += String.fromCharCode(code - 0xff21 + 0x41);
      continue;
    }
    if (code >= 0xff41 && code <= 0xff5a) {
      normalized += String.fromCharCode(code - 0xff41 + 0x41);
      continue;
    }
    if (code >= 0xff10 && code <= 0xff19) {
      normalized += String.fromCharCode(code - 0xff10 + 0x30);
      continue;
    }
    normalized += char;
  }
  return normalized.toLowerCase();
};

const removeWord = (value: string) =>
  REMOVE_WORDS.reduce((current, word) => current.replaceAll(word, ""), value);

const splitPath = (filepath: string) =>
  filepath.split(/[\\/]/).filter((part) => part.length > 0);

const fileStem = (filepath: string) => {
  const filename = splitPath(filepath).at(-1) ?? filepath;
  const lastDot = filename.lastIndexOf(".");
  return lastDot > 0 ? filename.slice(0, lastDot) : filename;
};

const extension = (filepath: string) => {
  const filename = splitPath(filepath).at(-1) ?? filepath;
  const lastDot = filename.lastIndexOf(".");
  return lastDot > 0 ? filename.slice(lastDot + 1).toLowerCase() : "";
};

const isNotGameFilename = (normalizedStem: string) =>
  STRICT_NOT_GAME_TERMS.some((term) =>
    normalizedStem.includes(normalizeForGameMatch(term)),
  );

const normalizedGames: NormalizedGameCache[] = allGameCaches.map((cache) => ({
  ...cache,
  normalizedName: normalizeForGameMatch(cache.gamename),
}));

const onpDistance = (left: string, right: string) => {
  let a = Array.from(left);
  let b = Array.from(right);
  let m = a.length;
  let n = b.length;

  if (m > n) {
    [a, b] = [b, a];
    [m, n] = [n, m];
  }

  const offset = m + 1;
  const delta = n - m;
  const fp = Array(m + n + 3).fill(-1);
  const snake = (k: number, yStart: number) => {
    let x = yStart - k;
    let y = yStart;
    while (x < m && y < n && a[x] === b[y]) {
      x += 1;
      y += 1;
    }
    return y;
  };

  for (let p = 0; ; p += 1) {
    for (let k = -p; k <= delta - 1; k += 1) {
      fp[k + offset] = snake(
        k,
        Math.max(fp[k - 1 + offset] + 1, fp[k + 1 + offset]),
      );
    }
    for (let k = delta + p; k >= delta + 1; k -= 1) {
      fp[k + offset] = snake(
        k,
        Math.max(fp[k - 1 + offset] + 1, fp[k + 1 + offset]),
      );
    }
    fp[delta + offset] = snake(
      delta,
      Math.max(fp[delta - 1 + offset] + 1, fp[delta + 1 + offset]),
    );
    if (fp[delta + offset] === n) {
      return delta + 2 * p;
    }
  }
};

const getComparableDistance = (left: string, right: string) => {
  const maxLength = Math.max(
    new TextEncoder().encode(left).length,
    new TextEncoder().encode(right).length,
  );
  if (!maxLength) {
    return 1;
  }
  return 1 - onpDistance(left, right) / maxLength;
};

const getPathSpecificGameId = (
  grandparent: string,
  parent: string,
) =>
  PATH_SPECIFIC_GAME_IDS.find(
    (rule) =>
      normalizeForGameMatch(rule.grandparent) === grandparent &&
      normalizeForGameMatch(rule.parent) === parent,
  )?.id ?? null;

const pushForcedGame = (
  scored: Array<{ cache: AllGameCacheOne; score: number }>,
  id: number,
  score: number,
) => {
  const forced = getGameCacheById(id);
  if (forced) {
    scored.push({ cache: forced, score });
  }
};

export const getGameCandidatesByFilePath = (
  filepath: string,
  threshold = 0.2,
  limit = 5,
): AllGameCacheOne[] => {
  const parts = splitPath(filepath);
  const normalizedStem = normalizeForGameMatch(fileStem(filepath));
  if (!normalizedStem || isNotGameFilename(normalizedStem)) {
    return [];
  }

  const shouldSkipFilename =
    ENGINE_NAMES.has(normalizedStem) ||
    normalizedStem === "game" ||
    normalizedStem === "start";
  const stem = removeWord(normalizedStem);
  const parent = normalizeForGameMatch(parts.at(-2) ?? "");
  const grandparent = normalizeForGameMatch(parts.at(-3) ?? "");
  const scored: Array<{ cache: AllGameCacheOne; score: number }> = [];

  const pathSpecificId = getPathSpecificGameId(grandparent, parent);
  if (pathSpecificId) {
    pushForcedGame(scored, pathSpecificId, 1000);
  }

  const forcedId = EQUAL_FILENAME_GAME_ID.get(stem);
  if (forcedId) {
    pushForcedGame(scored, forcedId, 100);
  }

  for (const game of normalizedGames) {
    if (IGNORE_GAME_ID.has(game.id)) {
      continue;
    }

    let score = 0;
    if (!shouldSkipFilename) {
      score = Math.max(score, getComparableDistance(stem, game.normalizedName));
    }
    score = Math.max(score, getComparableDistance(parent, game.normalizedName));
    if (grandparent) {
      score = Math.max(score, getComparableDistance(grandparent, game.normalizedName));
    }
    if (score > threshold) {
      scored.push({ cache: game, score });
    }
  }

  if (!scored.length) {
    for (const game of normalizedGames) {
      if (stem.length > 5 && game.normalizedName.includes(stem)) {
        scored.push({ cache: game, score: stem.length });
      }
      if (parent.length > 5 && game.normalizedName.includes(parent)) {
        scored.push({ cache: game, score: parent.length });
      }
    }
  }

  return scored
    .sort((a, b) => b.score - a.score)
    .slice(0, limit)
    .map(({ cache }) => ({
      id: cache.id,
      gamename: cache.gamename,
      thumbnailUrl: cache.thumbnailUrl,
    }));
};

export const getMostProbableGameByFilePath = (filepath: string) =>
  getGameCandidatesByFilePath(filepath, 0.8, 1)[0] ?? null;

const getConflictComparableStem = (path: string) =>
  fileStem(normalizeForGameMatch(path));

const shouldReplacePath = (
  currentPath: string,
  nextPath: string,
  normalizedGameName: string,
) => {
  const currentStem = getConflictComparableStem(currentPath);
  const nextStem = getConflictComparableStem(nextPath);
  let mustUpdate = false;
  let mustKeepCurrent = false;

  for (const ignoreWord of IGNORE_WORD_WHEN_CONFLICT) {
    const normalizedWord = normalizeForGameMatch(ignoreWord);
    if (currentStem.includes(normalizedWord)) {
      mustUpdate = true;
      break;
    }
    if (nextStem.includes(normalizedWord)) {
      mustKeepCurrent = true;
      break;
    }
  }

  for (const updateWord of SHOULD_UPDATE_WORD_WHEN_CONFLICT) {
    const normalizedWord = normalizeForGameMatch(updateWord);
    if (currentStem.includes(normalizedWord)) {
      mustKeepCurrent = true;
      break;
    }
    if (nextStem.includes(normalizedWord)) {
      mustUpdate = true;
      break;
    }
  }

  if (mustUpdate && !mustKeepCurrent) {
    return true;
  }
  if (mustKeepCurrent) {
    return false;
  }

  return (
    getComparableDistance(currentStem, normalizedGameName) <
    getComparableDistance(nextStem, normalizedGameName)
  );
};

export const getBestGamePathMatches = async (
  files: string[],
  onProcessed?: () => Promise<void> | void,
) => {
  const matches = new Map<number, DemoMatchedGamePath>();

  for (const path of files) {
    const cache = getMostProbableGameByFilePath(path);
    if (onProcessed) {
      await onProcessed();
    }
    if (!cache) {
      continue;
    }

    const current = matches.get(cache.id);
    if (
      !current ||
      shouldReplacePath(
        current.path,
        path,
        normalizeForGameMatch(cache.gamename),
      )
    ) {
      matches.set(cache.id, { cache, path });
    }
  }

  return matches;
};

export const isSupportedGamePath = (filepath: string) => {
  const ext = extension(filepath);
  return ext === "exe" || ext === "lnk";
};
