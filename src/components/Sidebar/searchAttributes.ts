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

// プレイ状況のキーを定義
const PLAY_STATUS_KEYS: AttributeKey[] = [ATTRIBUTES.UNPLAYED, ATTRIBUTES.PLAYING, ATTRIBUTES.CLEARED];

const EXPECTED_KEYS = Object.values(ATTRIBUTES);
// INITIAL_ATTRIBUTES は Object.values を使うと定義順になるので、
// ATTRIBUTES の定義順をプレイ状況が先頭になるように変更済み
const INITIAL_ATTRIBUTES = EXPECTED_KEYS.map((v) => ({
  key: v,
  enabled: false,
}));


export type Attribute = { key: AttributeKey; enabled: boolean };

// 属性をソートするための比較関数
const sortAttributes = (a: Attribute, b: Attribute): number => {
  const isAPlayStatus = PLAY_STATUS_KEYS.includes(a.key);
  const isBPlayStatus = PLAY_STATUS_KEYS.includes(b.key);

  // 1. プレイ状況属性を最優先
  if (isAPlayStatus && !isBPlayStatus) return -1;
  if (!isAPlayStatus && isBPlayStatus) return 1;

  // 2. プレイ状況属性同士、またはそれ以外の属性同士の場合：有効なものを優先
  if (a.enabled && !b.enabled) return -1;
  if (!a.enabled && b.enabled) return 1;

  // 3. 上記以外（例：両方プレイ状況かつ両方有効）は元の順序を維持（ここではキーの文字列順）
  //    より厳密な初期順序を維持したい場合は、INITIAL_ATTRIBUTESのインデックスを使う
  const indexOfA = EXPECTED_KEYS.indexOf(a.key);
  const indexOfB = EXPECTED_KEYS.indexOf(b.key);
  return indexOfA - indexOfB;
};


export const searchAttributes = () => {
  let storedValue = localStorage.getItem("search-attributes");
  let initialOrStoredAttributes = [...INITIAL_ATTRIBUTES]; // 配列をコピーして使用

  if (storedValue) {
    try {
      const parsed = JSON.parse(storedValue) as Attribute[];
      const allKeysValid = parsed.length === EXPECTED_KEYS.length && parsed.every(attr => EXPECTED_KEYS.includes(attr.key));

      if (allKeysValid) {
        // localStorageの値を使うが、定義順にソートし直すことで、
        // 新しい属性が追加された場合にも対応しやすくする
        const storedMap = new Map(parsed.map(attr => [attr.key, attr.enabled]));
        initialOrStoredAttributes = EXPECTED_KEYS.map(key => ({
            key,
            enabled: storedMap.get(key) || false, // localStorageになければfalse
        })).sort(sortAttributes);
      } else {
        console.warn("LocalStorage 'search-attributes' has unexpected structure or keys. Resetting to initial value and sorting.");
        initialOrStoredAttributes.sort(sortAttributes); // 初期値をソート
        localStorage.setItem("search-attributes", JSON.stringify(initialOrStoredAttributes));
      }
    } catch (e) {
      console.error("Failed to parse 'search-attributes' from localStorage. Resetting and sorting.", e);
      initialOrStoredAttributes.sort(sortAttributes); // 初期値をソート
      localStorage.setItem("search-attributes", JSON.stringify(initialOrStoredAttributes));
    }
  } else {
    initialOrStoredAttributes.sort(sortAttributes); // 初期値をソート
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
      return updatedAttrs.sort(sortAttributes); // 状態変更後もソート
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
