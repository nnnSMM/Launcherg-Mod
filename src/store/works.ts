import { getWorkByScrape } from "@/lib/scrapeWork";
import type { Work } from "@/lib/types";
import { createLocalStorageCache } from "@/lib/utils";

const WORKS_CACHE_VERSION = 1;
const WORKS_CACHE_TTL_MS = 1000 * 60 * 60 * 24 * 7;

const createWorks = () => {
  const getter = createLocalStorageCache<number, Work>(
    "works-cache",
    getWorkByScrape,
    {
      version: WORKS_CACHE_VERSION,
      invalidateMilliseconds: WORKS_CACHE_TTL_MS,
    }
  );

  return {
    get: getter,
  };
};

export const works = createWorks();
