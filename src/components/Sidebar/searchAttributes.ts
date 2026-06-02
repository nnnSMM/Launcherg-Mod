import type { CollectionElement } from "@/lib/types";
import { PlayStatus } from "@/lib/types";
import {
  createLocalStorageWritable,
  readLocalStorageJson,
  writeLocalStorageJson,
} from "@/lib/utils";

export const ATTRIBUTES = {
  UNPLAYED: "unplayed",
  PLAYING: "playing",
  CLEARED: "cleared",
  INTERRUPTED: "interrupted",
  NUKIGE: "nukige",
  NOT_NUKIGE: "not_nukige",
  LIKE: "like",
  EXIST_PATH: "exist_path",
} as const;

export const ATTRIBUTE_LABELS: { [key in AttributeKey]: string } = {
  unplayed: "未プレイ",
  playing: "プレイ中",
  cleared: "クリア済み",
  interrupted: "中断",
  nukige: "抜きゲー",
  not_nukige: "非抜きゲー",
  like: "お気に入り",
  exist_path: "パスが存在",
} as const;

export type AttributeKey = (typeof ATTRIBUTES)[keyof typeof ATTRIBUTES];
export type Attribute = { key: AttributeKey; enabled: boolean };

export const PLAY_STATUS_KEYS: AttributeKey[] = [
  ATTRIBUTES.UNPLAYED,
  ATTRIBUTES.PLAYING,
  ATTRIBUTES.CLEARED,
  ATTRIBUTES.INTERRUPTED,
];
const NUKIGE_KEYS: AttributeKey[] = [ATTRIBUTES.NUKIGE, ATTRIBUTES.NOT_NUKIGE]; // 抜きゲー関連キーを追加

const EXPECTED_KEYS = Object.values(ATTRIBUTES);
const INITIAL_ATTRIBUTES: Attribute[] = EXPECTED_KEYS.map((v) => ({
  key: v,
  enabled: false,
}));

const sortAttributes = (a: Attribute, b: Attribute): number => {
  const indexOfA = EXPECTED_KEYS.indexOf(a.key);
  const indexOfB = EXPECTED_KEYS.indexOf(b.key);
  return indexOfA - indexOfB;
};

const createInitialAttributes = (): Attribute[] =>
  [...INITIAL_ATTRIBUTES].sort(sortAttributes);

const isSearchAttribute = (value: unknown): value is Attribute => {
  if (!value || typeof value !== "object") {
    return false;
  }
  const candidate = value as Partial<Attribute>;
  return (
    EXPECTED_KEYS.includes(candidate.key as AttributeKey) &&
    typeof candidate.enabled === "boolean"
  );
};

export const normalizeSearchAttributes = (value: unknown): Attribute[] => {
  if (!Array.isArray(value) || value.length !== EXPECTED_KEYS.length) {
    return createInitialAttributes();
  }

  if (!value.every(isSearchAttribute)) {
    return createInitialAttributes();
  }

  const storedMap = new Map<AttributeKey, boolean>(
    value.map((attribute) => [attribute.key, attribute.enabled]),
  );
  if (storedMap.size !== EXPECTED_KEYS.length) {
    return createInitialAttributes();
  }

  return EXPECTED_KEYS.map((key) => ({
    key,
    enabled: storedMap.get(key) ?? false,
  })).sort(sortAttributes);
};

export const searchAttributes = () => {
  const initialOrStoredAttributes = normalizeSearchAttributes(
    readLocalStorageJson("search-attributes", INITIAL_ATTRIBUTES),
  );
  writeLocalStorageJson("search-attributes", initialOrStoredAttributes);

  const [attributes, getAttributes] = createLocalStorageWritable<Attribute[]>(
    "search-attributes",
    initialOrStoredAttributes
  );

  const toggleEnable = (keyToToggle: AttributeKey) => {
    attributes.update((currentAttrs) => {
      // まず、クリックされた属性の enabled 状態をトグル（反転）するかどうかを決定
      // すでに有効な同じボタンをクリックした場合は無効に、無効なボタンをクリックした場合は有効にする
      const currentlyEnabled = currentAttrs.find(attr => attr.key === keyToToggle)?.enabled || false;
      const newEnabledState = !currentlyEnabled;

      let updatedAttrs = currentAttrs.map(attr => {
        // プレイ状況の排他制御
        if (PLAY_STATUS_KEYS.includes(keyToToggle) && PLAY_STATUS_KEYS.includes(attr.key)) {
          return { ...attr, enabled: attr.key === keyToToggle ? newEnabledState : false };
        }
        // 抜きゲー関連の排他制御
        if (NUKIGE_KEYS.includes(keyToToggle) && NUKIGE_KEYS.includes(attr.key)) {
          return { ...attr, enabled: attr.key === keyToToggle ? newEnabledState : false };
        }
        // それ以外の属性、またはグループ外の属性で、今回トグル対象のキーの場合
        if (attr.key === keyToToggle) {
            return { ...attr, enabled: newEnabledState };
        }
        // それ以外の属性はそのまま
        return attr;
      });

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

export const matchesAttribute = (
  element: CollectionElement,
  key: AttributeKey,
): boolean => {
  switch (key) {
    case "nukige":
      return element.isNukige;
    case "not_nukige":
      return !element.isNukige;
    case "exist_path":
      return !!element.installAt;
    case "like":
      return !!element.likeAt;
    case "unplayed":
      return element.playStatus === PlayStatus.Unplayed;
    case "playing":
      return element.playStatus === PlayStatus.Playing;
    case "cleared":
      return element.playStatus === PlayStatus.Cleared;
    case "interrupted":
      return (
        element.playStatus === PlayStatus.Interrupted ||
        element.playStatus === PlayStatus.LegacyShelved
      );
  }
};

export const FILTER_BY_ATTRIBUTE: {
  [key in AttributeKey]: (src: CollectionElement[]) => CollectionElement[];
} = {
  nukige: (src) => src.filter((v) => matchesAttribute(v, "nukige")),
  not_nukige: (src) => src.filter((v) => matchesAttribute(v, "not_nukige")),
  exist_path: (src) => src.filter((v) => matchesAttribute(v, "exist_path")),
  like: (src) => src.filter((v) => matchesAttribute(v, "like")),
  unplayed: (src) => src.filter((v) => matchesAttribute(v, "unplayed")),
  playing: (src) => src.filter((v) => matchesAttribute(v, "playing")),
  cleared: (src) => src.filter((v) => matchesAttribute(v, "cleared")),
  interrupted: (src) => src.filter((v) => matchesAttribute(v, "interrupted")),
};
