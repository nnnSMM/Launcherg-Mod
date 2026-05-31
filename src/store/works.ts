import { getWorkByScrape } from "@/lib/scrapeWork";
import { commandGetAllElements } from "@/lib/command";
import type { Work } from "@/lib/types";
import { createLocalStorageCache } from "@/lib/utils";

const WORKS_CACHE_VERSION = 8;
const WORKS_CACHE_TTL_MS = 1000 * 60 * 60 * 24 * 7;
const DESCRIPTION_PREFETCH_DELAY_MS = 500;

const hasStoryDescription = (work: Work | undefined) =>
  !!work?.description?.trim();

const wait = (milliseconds: number) =>
  new Promise((resolve) => setTimeout(resolve, milliseconds));

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
    ensureRegisteredStories: async () => {
      let elements;
      try {
        elements = await commandGetAllElements();
      } catch (error) {
        console.warn("[works] failed to load registered games", error);
        return;
      }

      for (const element of elements) {
        if (hasStoryDescription(getter.peek(element.id))) {
          continue;
        }

        try {
          getter.setValue(element.id, await getWorkByScrape(element.id));
        } catch (error) {
          console.warn(
            `[works] failed to fetch missing story for ${element.id}`,
            error
          );
        }
        await wait(DESCRIPTION_PREFETCH_DELAY_MS);
      }
    },
  };
};

export const works = createWorks();
