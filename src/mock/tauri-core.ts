import type { CollectionElement, Work } from "@/lib/types";
import demoData from "@/mock/demoData.json";
import worksData from "@/mock/worksData.json";

const collectionElements =
  demoData.collectionElements as unknown as CollectionElement[];
const worksById = worksData as Record<number, Work>;
const works = Object.values(worksById);

const seiyaUrlByNamePart: [string, string][] = [
  ["紙の上の魔法使い", "https://seiya-saiga.com/game/uguisukagura/kamimaho.html"],
  ["創作彼女", "https://seiya-saiga.com/game/ainolinks/soukano.html"],
  ["ディメンション", "https://seiya-saiga.com/game/crystalia/dimensionlovers.html"],
];

export const invoke = async <T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> => {
  console.log("[Mock Tauri Core] invoke:", cmd, args);

  if (cmd === "get_all_elements") {
    return collectionElements as T;
  }

  if (cmd === "get_all_works") {
    return works as T;
  }

  if (cmd === "get_collection_element") {
    const id = Number(args?.collectionElementId);
    return (
      collectionElements.find((game) => Number(game.id) === id) ??
      collectionElements[0]
    ) as T;
  }

  if (cmd === "get_nearest_key_and_distance") {
    const name = String(args?.key ?? "");
    const result = seiyaUrlByNamePart.find(([namePart]) =>
      name.includes(namePart),
    );
    return (result ? [result[1], 0] : ["", 100]) as T;
  }

  if (cmd === "get_app_setting") {
    if (args?.key === "shortcut_game_id") {
      return "39837" as T;
    }
    return null as T;
  }

  if (cmd === "get_game_cache_all") {
    return works.map((work) => ({
      id: work.id,
      gamename: work.name,
      thumbnailUrl: work.imgUrl,
    })) as T;
  }

  if (cmd === "get_all_game_cache_last_updated") {
    return [works.length, "2026-05-17T00:00:00Z"] as T;
  }

  if (cmd === "get_pause_state") return false as T;
  if (cmd === "get_not_registered_detail_element_ids") return [] as T;

  if (cmd === "get_game_cache_by_id") {
    const id = Number(args?.id);
    const work = worksById[id];
    if (!work) return null as T;
    return {
      id: work.id,
      gamename: work.name,
      gamenameRuby: "",
      brandname: work.brandName,
      brandnameRuby: "",
      sellday: work.sellday,
      isNukige: false,
    } as T;
  }

  if (cmd === "get_game_screenshots") return [] as T;
  return null as T;
};

export const convertFileSrc = (p: string) => {
  if (p && p.startsWith("http")) return p;
  if (p && p.startsWith("demo-images/")) return p;
  return "images/dummy_thumbnail.svg";
};
