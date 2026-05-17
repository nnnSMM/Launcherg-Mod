import type { Work } from "@/lib/types";
import worksData from "@/mock/worksData.json";

const WORKS_DATA = worksData as Record<number, Work>;

const createWorks = () => {
  try {
    localStorage.removeItem("works-cache");
  } catch (e) {
    // ignore
  }

  const get = async (id: number): Promise<Work | null> => {
    const result = WORKS_DATA[id] ?? null;
    console.log(`[Mock Works] get(${id}) =>`, result ? result.name : "null");
    return result;
  };
  return { get };
};

export const works = createWorks();
