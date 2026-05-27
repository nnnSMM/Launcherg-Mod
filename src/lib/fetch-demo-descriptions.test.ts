import { describe, expect, it, vi } from "vitest";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Work } from "@/lib/types";

type DemoWorkMapping = {
  id: number;
  name: string;
  fanzaId?: string;
  dlsiteId?: string;
  dlsiteDomain?: string;
  steamAppId?: string;
  officialUrl?: string;
};

const { demoWorks, demoWorksById } = vi.hoisted(() => {
  const demoWorks: DemoWorkMapping[] = [
    { id: 20988, name: "紙の上の魔法使い", fanzaId: "nightingale_0001" },
    { id: 25861, name: "金色ラブリッチェ", fanzaId: "vsat_0229" },
    {
      id: 26245,
      name: "さくら、もゆ。 -as the Night's, Reincarnation-",
      fanzaId: "favorite_0011",
    },
    {
      id: 27059,
      name: "アメイジング・グレイス -What color is your attribute?-",
      fanzaId: "cabbage_0003",
    },
    { id: 38696, name: "記憶の鍵盤", steamAppId: "3782920" },
    { id: 38631, name: "魔法少女ノ魔女裁判", steamAppId: "3101040" },
    { id: 28941, name: "白昼夢の青写真", fanzaId: "hobe_0494" },
    { id: 30122, name: "冥契のルペルカリア", fanzaId: "nightingale_0005" },
    { id: 31106, name: "創作彼女の恋愛公式", fanzaId: "akbs_0127" },
    {
      id: 31597,
      name: "終のステラ",
      fanzaId: "vsat_0288",
      dlsiteId: "VJ015604",
      dlsiteDomain: "pro",
    },
    { id: 38794, name: "ライムライト・レモネードジャム", fanzaId: "yuzu_0012" },
    { id: 39837, name: "ディメンション凸ラバース!!", fanzaId: "spal_0201" },
  ];

  return {
    demoWorks,
    demoWorksById: new Map(demoWorks.map((work) => [work.id, work])),
  };
});

vi.mock("@/lib/scrapeSql", () => {
  return {
    scrapeSql: async (query: string, colNums: number) => {
      if (colNums !== 5 || !/\bfrom\s+gamelist\b/i.test(query)) {
        return [];
      }

      const id = Number(query.match(/\bwhere\s+id\s*=\s*(\d+)/i)?.[1] ?? 0);
      const work = demoWorksById.get(id);
      if (!work) {
        return [];
      }

      const dlsiteDomain =
        work.dlsiteDomain ??
        (work.dlsiteId
          ? work.dlsiteId.toUpperCase().startsWith("VJ")
            ? "pro"
            : "maniax"
          : "");

      return [
        [
          work.fanzaId ?? "",
          work.dlsiteId ?? "",
          dlsiteDomain,
          work.steamAppId ?? "",
          work.officialUrl ?? "",
        ],
      ];
    },
  };
});

vi.mock("@tauri-apps/plugin-http", () => {
  return {
    fetch: async (url: string | URL, init?: RequestInit) => {
      const urlString = String(url);
      const headers = new Headers(init?.headers);

      if (urlString.includes("dmm.co.jp") || urlString.includes("dlsoft.dmm.co.jp")) {
        headers.set(
          "Cookie",
          "age_check_done=1; age_check_new_origin=1; ckcy=1; guest_id=launchergdemo"
        );
        headers.set(
          "User-Agent",
          "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
        );
      } else if (urlString.includes("dlsite.com")) {
        headers.set("Cookie", "adultchecked=1; locale=ja_JP;");
        headers.set(
          "User-Agent",
          "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
        );
        headers.set("Accept-Language", "ja,en;q=0.9");
      }

      return globalThis.fetch(urlString, { ...init, headers });
    },
  };
});

import { getWorkByScrape } from "@/lib/scrapeWork";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WORKS_DATA_PATH = path.resolve(__dirname, "../mock/worksData.json");

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const assertCompleteDemoWork = (work: Work, expected: DemoWorkMapping) => {
  const requiredFields = {
    name: work.name,
    brandName: work.brandName,
    sellday: work.sellday,
    imgUrl: work.imgUrl,
    officialHomePage: work.officialHomePage,
    description: work.description,
  };

  const missingFields = Object.entries(requiredFields)
    .filter(([, value]) => !String(value ?? "").trim())
    .map(([field]) => field);

  if (missingFields.length) {
    throw new Error(
      `Game ${expected.id} (${expected.name}) is missing: ${missingFields.join(", ")}`
    );
  }

  if (
    !Array.isArray(work.creators.illustrators) ||
    !Array.isArray(work.creators.writers) ||
    !Array.isArray(work.creators.voiceActors) ||
    !Array.isArray(work.musics)
  ) {
    throw new Error(`Game ${expected.id} (${expected.name}) has invalid creator/music arrays`);
  }
};

describe("fetch demo works", () => {
  it("rebuilds worksData.json from Web using the app scraping algorithm", async () => {
    console.log("Starting scraping demo works using getWorkByScrape...");

    const worksData: Record<string, Work> = {};
    const failures: string[] = [];

    for (const [index, workInfo] of demoWorks.entries()) {
      console.log(
        `Scraping ${index + 1}/${demoWorks.length}: Game ${workInfo.id} (${workInfo.name})`
      );

      try {
        const work = await getWorkByScrape(workInfo.id);
        assertCompleteDemoWork(work, workInfo);
        worksData[String(workInfo.id)] = work;
        console.log(`Updated ${work.name}`);
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        failures.push(`Game ${workInfo.id} (${workInfo.name}): ${message}`);
        console.error(`Failed to scrape Game ${workInfo.id} (${workInfo.name})`, error);
      }

      if (index < demoWorks.length - 1) {
        await sleep(1500);
      }
    }

    if (failures.length) {
      throw new Error(`Failed to update demo works:\n${failures.join("\n")}`);
    }

    expect(Object.keys(worksData)).toHaveLength(demoWorks.length);
    fs.writeFileSync(WORKS_DATA_PATH, `${JSON.stringify(worksData, null, 2)}\n`, "utf-8");
    console.log(`Successfully rebuilt ${demoWorks.length} works in worksData.json`);
  }, 180000);
});
