export type Option<T> = { label: string; value: T; otherLabels?: string[] };

import type { CollectionElement } from "@/lib/types";
import { readable, writable, type Readable, type Writable } from "svelte/store";
import { toHiragana, toRomaji } from "wanakana";

export const collectionElementsToOptions = (elements: CollectionElement[]) =>
  elements.map((v) => ({
    label: v.gamename,
    value: v.id,
    otherLabels: [
      toHiragana(v.gamenameRuby),
      toRomaji(v.gamenameRuby),
      v.brandname,
      toHiragana(v.brandnameRuby),
      toRomaji(v.brandnameRuby),
    ],
  }));

export const useFilter = <T>(
  query: Writable<string>,
  options: Readable<Option<T>[]>,
  getOptions: () => Option<T>[]
) => {
  const filtered = readable<Option<T>[]>([...getOptions()], (set) => {
    const cache = new Map<string, Option<T>[]>();
    let latestQuery = "";
    let lazyQueryTimer: ReturnType<typeof setTimeout> | null = null;

    const applyFilter = (rawQuery: string) => {
      const normalizedQuery = rawQuery.trim().toLowerCase();
      if (!normalizedQuery) {
        set([...getOptions()]);
        return;
      }

      const cached = Array.from(cache.entries()).find(([input]) =>
        normalizedQuery.includes(input),
      );
      const targetOptions = cached ? cached[1] : getOptions();
      const nextFiltered = targetOptions.filter((option) =>
        [option.label, ...(option.otherLabels ?? [])].some((key) =>
          key.toLowerCase().includes(normalizedQuery),
        ),
      );
      cache.set(normalizedQuery, nextFiltered);
      set(nextFiltered);
    };

    const unsubscribeQuery = query.subscribe((nextQuery) => {
      latestQuery = nextQuery;
      if (lazyQueryTimer) {
        clearTimeout(lazyQueryTimer);
      }
      lazyQueryTimer = setTimeout(() => {
        applyFilter(latestQuery);
        lazyQueryTimer = null;
      }, 200);
    });

    const unsubscribeOptions = options.subscribe(() => {
      cache.clear();
      applyFilter(latestQuery);
    });

    return () => {
      if (lazyQueryTimer) {
        clearTimeout(lazyQueryTimer);
      }
      unsubscribeQuery();
      unsubscribeOptions();
    };
  });

  return { query, filtered };
};
