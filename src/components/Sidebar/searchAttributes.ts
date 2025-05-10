import type { CollectionElement } from "@/lib/types";
import { PlayStatus } from "@/lib/types";
import { createLocalStorageWritable } from "@/lib/utils";

export const ATTRIBUTES = {
  // プレイ状況を先頭に定義
  UNPLAYED: "unplayed",
  PLAYING: "playing",
  CLEARED: "cleared",
  // それ以外の属性
  NUKIGE: "nukige",
  NOT_NUKIGE: "not_nukige",
  LIKE: "like",
  EXIST_PATH: "exist_path",
} as const;

export const ATTRIBUTE_LABELS: { [key in AttributeKey]: string } = {
  unplayed: "未プレイ",
  playing: "プレイ中",
  cleared: "クリア済み",
  nukige: "抜きゲー",
  not_nukige: "非抜きゲー",
  like: "お気に入り",
  exist_path: "パスが存在",
} as const;

export type AttributeKey = (typeof ATTRIBUTES)[keyof typeof ATTRIBUTES];

// プレイ状況のキーを定義し、エクスポートする
export const PLAY_STATUS_KEYS: AttributeKey[] = [ATTRIBUTES.UNPLAYED, ATTRIBUTES.PLAYING, ATTRIBUTES.CLEARED];

const EXPECTED_KEYS = Object.values(ATTRIBUTES);
const INITIAL_ATTRIBUTES = EXPECTED_KEYS.map((v) => ({
  key: v,
  enabled: false,
}));


export type Attribute = { key: AttributeKey; enabled: boolean };

const sortAttributes = (a: Attribute, b: Attribute): number => {
  const indexOfA = EXPECTED_KEYS.indexOf(a.key);
  const indexOfB = EXPECTED_KEYS.indexOf(b.key);
  return indexOfA - indexOfB;
};


export const searchAttributes = () => {
  let storedValue = localStorage.getItem("search-attributes");
  let initialOrStoredAttributes = [...INITIAL_ATTRIBUTES];

  if (storedValue) {
    try {
      const parsed = JSON.parse(storedValue) as Attribute[];
      const allKeysValid = parsed.length === EXPECTED_KEYS.length && parsed.every(attr => EXPECTED_KEYS.includes(attr.key));

      if (allKeysValid) {
        const storedMap = new Map(parsed.map(attr => [attr.key, attr.enabled]));
        initialOrStoredAttributes = EXPECTED_KEYS.map(key => ({
            key,
            enabled: storedMap.get(key) || false,
        })).sort(sortAttributes);
      } else {
        console.warn("LocalStorage 'search-attributes' has unexpected structure or keys. Resetting to initial value and sorting.");
        initialOrStoredAttributes.sort(sortAttributes);
        localStorage.setItem("search-attributes", JSON.stringify(initialOrStoredAttributes));
      }
    } catch (e) {
      console.error("Failed to parse 'search-attributes' from localStorage. Resetting and sorting.", e);
      initialOrStoredAttributes.sort(sortAttributes);
      localStorage.setItem("search-attributes", JSON.stringify(initialOrStoredAttributes));
    }
  } else {
    initialOrStoredAttributes.sort(sortAttributes);
    localStorage.setItem("search-attributes", JSON.stringify(initialOrStoredAttributes));
  }

  const [attributes, getAttributes] = createLocalStorageWritable<Attribute[]>(
    "search-attributes",
    initialOrStoredAttributes
  );

  const toggleEnable = (key: AttributeKey) => {
    attributes.update((attrs) => {
      const updatedAttrs = attrs.map(attr =>
        attr.key === key ? { ...attr, enabled: !attr.enabled } : attr
      );
      return updatedAttrs.sort(sortAttributes);
    });
  };

  return {
    attributes: {
      subscribe: attributes.subscribe,
    },
    toggleEnable,
  };
};

export const FILTER_BY_ATTRIBUTE: {
  [key in AttributeKey]: (src: CollectionElement[]) => CollectionElement[];
} = {
  nukige: (src) => src.filter((v) => v.isNukige),
  not_nukige: (src) => src.filter((v) => !v.isNukige),
  exist_path: (src) => src.filter((v) => v.installAt),
  like: (src) => src.filter((v) => v.likeAt),
  unplayed: (src) => src.filter((v) => v.playStatus === PlayStatus.Unplayed),
  playing: (src) => src.filter((v) => v.playStatus === PlayStatus.Playing),
  cleared: (src) => src.filter((v) => v.playStatus === PlayStatus.Cleared),
};
