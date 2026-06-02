export type Option<T> = { label: string; value: T; otherLabels?: string[] };
import type { CollectionElement } from "@/lib/types";
import { isNotNullOrUndefined } from "@/lib/utils";
import { readable, writable, type Readable } from "svelte/store";
import TrieSearch from "trie-search";
import { toHiragana, toRomaji } from "wanakana";

type KeyValue<T> = {
  key: string;
  value: T;
};

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

export const useTrieFilter = <T>(
  options: Readable<Option<T>[]>,
  getOptions: () => Option<T>[]
) => {
  const query = writable("");
  const filtered = readable<Option<T>[]>([...getOptions()], (set) => {
    let latestQuery = "";
    let optionMap = new Map<Option<T>["value"], Option<T>>();
    let trie: TrieSearch<KeyValue<T>> = new TrieSearch<KeyValue<T>>("key");

    const rebuildIndex = () => {
      optionMap = new Map<Option<T>["value"], Option<T>>();
      trie = new TrieSearch<KeyValue<T>>("key");

      for (const option of getOptions()) {
        optionMap.set(option.value, option);
        trie.add({ key: option.label, value: option.value });
        for (const otherLabel of option.otherLabels ?? []) {
          trie.add({ key: otherLabel, value: option.value });
        }
      }
    };

    const applyFilter = (rawQuery: string) => {
      if (!rawQuery) {
        set([...getOptions()]);
        return;
      }
      const searched = trie.search(rawQuery);
      set(
        [...new Set(searched.map((v) => v.value))]
          .map((v) => optionMap.get(v))
          .filter(isNotNullOrUndefined),
      );
    };

    rebuildIndex();

    const unsubscribeQuery = query.subscribe((nextQuery) => {
      latestQuery = nextQuery;
      applyFilter(nextQuery);
    });

    const unsubscribeOptions = options.subscribe(() => {
      rebuildIndex();
      applyFilter(latestQuery);
    });

    return () => {
      unsubscribeQuery();
      unsubscribeOptions();
    };
  });

  return { query, filtered };
};
