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

  const [loadingStore] = createWritable<boolean>(true); // Initial loading state

  const refetch = async () => {
    loadingStore.set(true);
    try {
      const freshElements = await commandGetAllElements();
      set(freshElements);
    } finally {
      loadingStore.set(false);
    }
  };

  const updateLike = (id: number, isLike: boolean) => {
    const now = new Date();
    const likeAt = `${now.getFullYear()}-${now.getMonth() + 1
      }-${now.getDate()}`;
    update((elements) =>
      elements.map((v) =>
        v.id === id ? { ...v, likeAt: isLike ? likeAt : null } : { ...v }
      )
    );
  };

  const updatePlayStatus = (id: number, playStatus: PlayStatus) => {
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
    updatePlayStatus,
    shown,
    loading: loadingStore, // Expose loading store
  };
}

export const sidebarCollectionElements = createSidebarCollectionElements();
