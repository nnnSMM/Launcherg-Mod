import type { Work } from "@/lib/types";
import { getWorkById } from "@/mock/demoCatalog";

const createWorks = () => {
  try {
    localStorage.removeItem("works-cache");
  } catch (e) {
    // ignore
  }

  const get = async (id: number): Promise<Work | null> => {
    const result = getWorkById(id);
    console.log(`[Mock Works] get(${id}) =>`, result ? result.name : "null");
    return result;
  };
  return {
    get,
    ensureRegisteredStories: async () => {
      // デモ環境ではストーリー詳細のプリフェッチは不要なため、即座に解決するダミー実装
    },
  };
};

export const works = createWorks();
