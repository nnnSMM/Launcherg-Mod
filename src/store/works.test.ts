import { beforeEach, describe, expect, it, vi } from "vitest";
import { commandGetAllElements } from "@/lib/command";
import { getWorkByScrape } from "@/lib/scrapeWork";
import type { CollectionElement, Work } from "@/lib/types";

vi.mock("@/lib/command", () => ({
  commandGetAllElements: vi.fn(),
}));

vi.mock("@/lib/scrapeWork", () => ({
  getWorkByScrape: vi.fn(),
}));

const work = (id: number, description?: string): Work => ({
  id,
  name: `work-${id}`,
  brandId: 1,
  brandName: "brand",
  description,
  officialHomePage: "",
  sellday: "2026-01-01",
  imgUrl: "",
  statistics: {
    median: 0,
    average: 0,
    count: 0,
    playTime: "0",
  },
  creators: {
    illustrators: [],
    writers: [],
    voiceActors: [],
  },
  musics: [],
});

const collectionElement = (id: number): CollectionElement => ({
  id,
  gamename: `game-${id}`,
  gamenameRuby: "",
  brandname: "brand",
  brandnameRuby: "",
  sellday: "2026-01-01",
  isNukige: false,
  installAt: null,
  firstPlayAt: null,
  lastPlayAt: null,
  likeAt: null,
  playStatus: 0,
  totalPlayTimeSeconds: 0,
  registeredAt: "2026-01-01",
  exePath: "",
  lnkPath: "",
  icon: "",
  thumbnail: "",
  thumbnailWidth: null,
  thumbnailHeight: null,
  updatedAt: "2026-01-01",
});

const loadWorks = async () => (await import("./works")).works;

describe("works.ensureRegisteredStories", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    localStorage.clear();
  });

  it("fetches stories only for registered works missing a cached description", async () => {
    localStorage.setItem(
      "works-cache",
      JSON.stringify({
        1: { value: work(1, "cached story"), createdAt: Date.now(), version: 8 },
        2: { value: work(2), createdAt: Date.now(), version: 8 },
        3: { value: work(3), createdAt: Date.now(), version: 8 },
      })
    );
    vi.mocked(commandGetAllElements).mockResolvedValue([
      collectionElement(1),
      collectionElement(2),
    ]);
    vi.mocked(getWorkByScrape).mockResolvedValue(work(2, "fetched story"));

    const works = await loadWorks();
    await works.ensureRegisteredStories();

    expect(getWorkByScrape).toHaveBeenCalledTimes(1);
    expect(getWorkByScrape).toHaveBeenCalledWith(2);
    expect(await works.get(2)).toMatchObject({ description: "fetched story" });
  });
});
