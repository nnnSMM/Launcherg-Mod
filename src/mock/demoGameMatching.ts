import type { AllGameCacheOne } from "@/lib/types";
import { allGameCaches, getGameCacheById } from "@/mock/demoCatalog";

type NormalizedGameCache = AllGameCacheOne & {
  normalizedName: string;
  grams: Set<string>;
};

const STRICT_NOT_GAME_TERMS = [
  "manual",
  "install",
  "uninstall",
  "unins",
  "autorun",
  "license",
  "setup",
  "config",
  "setting",
  "support",
  "update",
  "updchk",
  "bootstrap",
  "unitycrashhandler",
  "sigchk",
  "delfile",
  "マニュアル",
  "詳細設定",
  "はじめに",
  "サポート",
  "セーブデータ",
  "インストール",
  "アンインストール",
  "削除",
  "公式サイト",
  "ホームページ",
];

const ENGINE_NAMES = new Set(["bgi", "siglusengine", "nscripter", "game", "start"]);

const EQUAL_FILENAME_GAME_ID = new Map<string, number>([["pieces", 27123]]);

export const normalizeForGameMatch = (value: string) =>
  value
    .normalize("NFKC")
    .toLowerCase()
    .replace(/ゲームを起動|ゲームの起動|_単独動作版|ダウンロード版|dl版/gi, "")
    .replace(/[「」『』【】［］\[\]（）()_\-‐ー・＊*!.!！?？:：/\\\s]/g, "");

const gramsOf = (value: string) => {
  const grams = new Set<string>();
  if (value.length <= 2) {
    if (value) grams.add(value);
    return grams;
  }
  for (let i = 0; i < value.length - 1; i++) {
    grams.add(value.slice(i, i + 2));
  }
  return grams;
};

const normalizedGames: NormalizedGameCache[] = allGameCaches.map((cache) => {
  const normalizedName = normalizeForGameMatch(cache.gamename);
  return {
    ...cache,
    normalizedName,
    grams: gramsOf(normalizedName),
  };
});

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

const diceScore = (a: Set<string>, b: Set<string>) => {
  if (!a.size || !b.size) {
    return 0;
  }
  let intersection = 0;
  for (const gram of a) {
    if (b.has(gram)) {
      intersection++;
    }
  }
  return (2 * intersection) / (a.size + b.size);
};

const scoreSource = (source: string, sourceGrams: Set<string>, game: NormalizedGameCache) => {
  if (!source || !game.normalizedName) {
    return 0;
  }
  if (source === game.normalizedName) {
    return 1;
  }
  if (game.normalizedName.includes(source) && source.length >= 3) {
    return Math.min(0.99, 0.9 + Math.min(source.length, 20) * 0.001);
  }
  if (source.includes(game.normalizedName) && game.normalizedName.length >= 3) {
    return Math.min(0.99, 0.86 + (game.normalizedName.length / source.length) * 0.1);
  }
  if (source.length < 5) {
    return 0;
  }
  return diceScore(sourceGrams, game.grams);
};

export const getGameCandidatesByFilePath = (
  filepath: string,
  threshold = 0.2,
  limit = 5,
): AllGameCacheOne[] => {
  const parts = splitPath(filepath);
  const stem = normalizeForGameMatch(fileStem(filepath));
  if (!stem || isNotGameFilename(stem)) {
    return [];
  }

  const forcedId = EQUAL_FILENAME_GAME_ID.get(stem);
  if (forcedId) {
    const forced = getGameCacheById(forcedId);
    if (forced) {
      return [forced];
    }
  }

  const parent = normalizeForGameMatch(parts.at(-2) ?? "");
  const grandparent = normalizeForGameMatch(parts.at(-3) ?? "");
  const shouldSkipFilename = ENGINE_NAMES.has(stem);
  const sources = [...new Set([shouldSkipFilename ? "" : stem, parent, grandparent])].filter(
    Boolean,
  );
  const sourceGrams = new Map(sources.map((source) => [source, gramsOf(source)]));

  const scored: Array<{ cache: AllGameCacheOne; score: number }> = [];
  for (const game of normalizedGames) {
    let score = 0;
    for (const source of sources) {
      score = Math.max(score, scoreSource(source, sourceGrams.get(source) ?? new Set(), game));
    }
    if (score > threshold) {
      scored.push({ cache: game, score });
    }
  }

  return scored
    .sort((a, b) => b.score - a.score || a.cache.id - b.cache.id)
    .slice(0, limit)
    .map(({ cache }) => ({
      id: cache.id,
      gamename: cache.gamename,
      thumbnailUrl: cache.thumbnailUrl,
    }));
};

export const getMostProbableGameByFilePath = (filepath: string) =>
  getGameCandidatesByFilePath(filepath, 0.8, 1)[0] ?? null;

export const isSupportedGamePath = (filepath: string) => {
  const ext = extension(filepath);
  return ext === "exe" || ext === "lnk" || ext === "url";
};
