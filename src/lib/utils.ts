import { writable } from "svelte/store";

export function createWritable<T>(initialValue: T) {
  let _value = initialValue;
  const store = writable<T>(initialValue);
  store.subscribe((v) => {
    _value = v;
  });
  return [store, () => _value] as const;
}

export const localStorageWritable = <T>(key: string, initialValue: T) => {
  let stored = localStorage.getItem(key);
  const store = writable<T>(stored ? JSON.parse(stored) : initialValue);
  store.subscribe((value) => localStorage.setItem(key, JSON.stringify(value)));
  return store;
};

export const createLocalStorageWritable = <T>(key: string, initialValue: T) => {
  let _value = initialValue;
  const store = localStorageWritable<T>(key, initialValue);
  store.subscribe((v) => {
    _value = v;
  });
  return [store, () => _value] as const;
};

export type Cache<S extends string | number, U> = Record<
  S,
  { createdAt: number; value: U }
>;

export const createLocalStorageCache = <K extends string | number, T>(
  key: string,
  fetcher: (key: K) => Promise<T>,
  invalidateMilliseconds = 1000 * 60 * 60 * 24
) => {
  const initialValue = {} as Cache<K, T>;
  const [cache, getCache] = createLocalStorageWritable(key, initialValue);

  const getter = async (key: K): Promise<T> => {
    const now = new Date().getTime();
    const cached = getCache()[key];
    if (cached && now < cached.createdAt + invalidateMilliseconds) {
      return cached.value;
    }
    const value = await fetcher(key);
    cache.update((v) => {
      v[key] = { value: value, createdAt: now };
      return v;
    });
    return value;
  };
  return getter;
};

export const convertSpecialCharacters = (str: string) => {
  const tempElement = document.createElement("textarea");
  tempElement.innerHTML = str;
  const val = tempElement.value;
  tempElement.remove();
  return val;
};

export const isNotNullOrUndefined = <T>(
  src: T | null | undefined
): src is T => {
  return src !== null && src !== undefined;
};

export const rand = (max = 100000) => Math.floor(Math.random() * max);

export const formatPlayTime = (totalSeconds: number): string => {
  // 2時間未満 (分で表示)
  if (totalSeconds < 7200) {
    const minutes = Math.floor(totalSeconds / 60);
    return `${minutes}分`;
  }

  // 2時間以上 (時間で表示)
  const hours = totalSeconds / 3600;
  return `${Math.floor(hours * 10) / 10}時間`;
};


export const formatLastPlayed = (isoString: string | null | undefined): string => {
  if (!isoString) {
    return "";
  }

  const lastPlayedDate = new Date(isoString);
  const now = new Date();

  // 時間をリセットして日付のみで比較
  lastPlayedDate.setHours(0, 0, 0, 0);
  now.setHours(0, 0, 0, 0);

  const diffTime = now.getTime() - lastPlayedDate.getTime();
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    return "今日";
  }
  if (diffDays === 1) {
    return "昨日";
  }
  if (diffDays <= 14) {
    return `${diffDays}日前`;
  }
  
  // 14日以上前なら年月日を表示
  return new Date(isoString).toLocaleDateString("ja-JP");
};