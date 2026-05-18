import type { AllGameCacheOne, CollectionElement, Work } from "@/lib/types";
import allGamesData from "../../script/all_games.json";
import demoData from "@/mock/demoData.json";
import worksData from "@/mock/worksData.json";

type ScriptGameCache = {
  id: number;
  gamename: string;
  thumbnailUrl: string;
};

const scriptGames = allGamesData as ScriptGameCache[];
const demoElements =
  demoData.collectionElements as unknown as CollectionElement[];
const worksById = worksData as Record<number, Work>;

const cacheById = new Map<number, AllGameCacheOne>();

for (const cache of scriptGames) {
  cacheById.set(cache.id, cache);
}

for (const element of demoElements) {
  cacheById.set(element.id, {
    id: element.id,
    gamename: element.gamename,
    thumbnailUrl: element.thumbnail,
  });
}

for (const work of Object.values(worksById)) {
  cacheById.set(work.id, {
    id: work.id,
    gamename: work.name,
    thumbnailUrl: work.imgUrl,
  });
}

export const allGameCaches = Array.from(cacheById.values()).sort(
  (a, b) => a.id - b.id,
);

export const getGameCacheById = (id: number) => cacheById.get(id) ?? null;

const demoElementById = new Map(demoElements.map((element) => [element.id, element]));

export const getCollectionElementSeed = (id: number) =>
  demoElementById.get(id) ?? null;

export const getWorkById = (id: number): Work | null => {
  const work = worksById[id];
  if (work) {
    return work;
  }

  const cache = getGameCacheById(id);
  if (!cache) {
    return null;
  }

  const seed = getCollectionElementSeed(id);
  return {
    id: cache.id,
    name: cache.gamename,
    brandId: 0,
    brandName: seed?.brandname ?? "",
    officialHomePage: "",
    sellday: seed?.sellday ?? "",
    imgUrl: cache.thumbnailUrl,
    statistics: {
      median: 0,
      average: 0,
      count: 0,
      playTime: "",
    },
    creators: {
      illustrators: [],
      writers: [],
      voiceActors: [],
    },
    musics: [],
  };
};

export const getDetailSeedById = (id: number) => {
  const seed = getCollectionElementSeed(id);
  return {
    gamenameRuby: seed?.gamenameRuby ?? "",
    brandname: seed?.brandname ?? "",
    brandnameRuby: seed?.brandnameRuby ?? "",
    sellday: seed?.sellday ?? "",
    isNukige: seed?.isNukige ?? false,
  };
};
