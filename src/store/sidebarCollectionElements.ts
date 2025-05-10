import { commandGetAllElements } from "@/lib/command";
import type {
  CollectionElement,
  CollectionElementsWithLabel,
  PlayStatus, // 追加
} from "@/lib/types";
import { createWritable } from "@/lib/utils";

function createSidebarCollectionElements() {
  const [{ subscribe, update, set }, value] = createWritable<
    CollectionElement[]
  >([]);

  const refetch = async () => {
    set(await commandGetAllElements());
  };
  const updateLike = (id: number, isLike: boolean) => {
    const now = new Date();
    const likeAt = `${now.getFullYear()}-${
      now.getMonth() + 1
    }-${now.getDate()}`;
    update((elements) =>
      elements.map((v) =>
        v.id === id ? { ...v, likeAt: isLike ? likeAt : null } : { ...v }
      )
    );
  };

  const updatePlayStatus = (id: number, playStatus: PlayStatus) => { // 追加
    update((elements) =>
      elements.map((v) =>
        v.id === id ? { ...v, playStatus } : { ...v }
      )
    );
  };

  const [shown] = createWritable<CollectionElementsWithLabel[]>([]);

  return {
    subscribe,
    value,
    refetch,
    updateLike,
    updatePlayStatus, // 追加
    shown,
  };
}

export const sidebarCollectionElements = createSidebarCollectionElements();
