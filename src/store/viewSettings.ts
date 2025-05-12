import { localStorageWritable } from "@/lib/utils";
import type { SortOrder } from "@/components/Sidebar/sort";
import type { Attribute } from "@/components/Sidebar/searchAttributes";
import { searchAttributes as createSearchAttributesStore } from "@/components/Sidebar/searchAttributes";

export const currentSortOrder = localStorageWritable<SortOrder>("sort-order", "gamename-asc");

const { attributes: globalAttributesStore, toggleEnable: globalToggleAttributeEnable } = createSearchAttributesStore();
export const currentAttributes = globalAttributesStore;
export const toggleAttribute = globalToggleAttributeEnable;
