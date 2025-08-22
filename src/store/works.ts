import { getWorkByScrape } from "@/lib/scrapeWork";
import type { Work } from "@/lib/types";
import { createLocalStorageCache } from "@/lib/utils";

const createWorks = () => {
  const getter = createLocalStorageCache<number, Work>(
    "works-cache-v2",
    getWorkByScrape
  );

  return {
    get: getter,
  };
};

export const works = createWorks();
