import { describe, expect, it, vi } from "vitest";

vi.mock("@/lib/command", () => ({
  commandGetAppSetting: vi.fn(),
  commandGetVndbScreenshotCache: vi.fn(),
  commandUpsertVndbScreenshotCache: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-http", () => ({
  fetch: vi.fn(),
}));

import {
  createVndbRequestBody,
  filterVndbScreenshots,
  isFreshVndbCache,
  parseVndbScreenshots,
  readScreenshotsFromCache,
} from "./useVndbScreenshots";
import type { VndbScreenshot, VndbScreenshotCache } from "./types";
import type { CollectionElement } from "./types";

const screenshot = (overrides: Partial<VndbScreenshot> = {}): VndbScreenshot => ({
  id: "sf1",
  url: "https://example.com/full.jpg",
  thumbnail: "https://example.com/thumb.jpg",
  dims: [1280, 720],
  thumbnailDims: [320, 180],
  sexual: 0,
  violence: 0,
  languages: ["ja"],
  ...overrides,
});

const collectionElement = (
  overrides: Partial<CollectionElement> = {},
): CollectionElement => ({
  id: 1,
  gamename: "日本語タイトル",
  gamenameRuby: "にほんごたいとる",
  brandname: "テストブランド",
  brandnameRuby: "てすとぶらんど",
  sellday: "2024-05-10",
  isNukige: false,
  installAt: null,
  lastPlayAt: null,
  likeAt: null,
  playStatus: 0,
  totalPlayTimeSeconds: 0,
  registeredAt: "2026-05-10T00:00:00.000Z",
  exePath: "",
  lnkPath: "",
  icon: "",
  thumbnail: "",
  thumbnailWidth: null,
  thumbnailHeight: null,
  updatedAt: "2026-05-10T00:00:00.000Z",
  ...overrides,
});

describe("useVndbScreenshots", () => {
  it("extracts only Japanese-release screenshots from VNDB response", () => {
    const parsed = parseVndbScreenshots({
      results: [
        {
          id: "v1",
          title: "Romanized",
          alttitle: "日本語タイトル",
          screenshots: [
            {
              id: "sf1",
              url: "https://example.com/ja.jpg",
              thumbnail: "https://example.com/ja-t.jpg",
              release: { languages: [{ lang: "ja" }] },
            },
            {
              id: "sf2",
              url: "https://example.com/en.jpg",
              thumbnail: "https://example.com/en-t.jpg",
              release: { languages: [{ lang: "en" }] },
            },
            {
              id: "sf3",
              url: "https://example.com/mixed.jpg",
              thumbnail: "https://example.com/mixed-t.jpg",
              release: { languages: [{ lang: "ja" }, { lang: "en" }] },
            },
            {
              id: "sf4",
              url: "https://example.com/mtl.jpg",
              thumbnail: "https://example.com/mtl-t.jpg",
              release: { languages: [{ lang: "ja", mtl: true }] },
            },
            {
              id: "sf5",
              url: "https://example.com/unknown.jpg",
              thumbnail: "https://example.com/unknown-t.jpg",
            },
          ],
        },
      ],
    });

    expect(parsed.status).toBe("ok");
    expect(parsed.vndbId).toBe("v1");
    expect(parsed.matchedTitle).toBe("日本語タイトル");
    expect(parsed.screenshots).toHaveLength(1);
    expect(parsed.screenshots[0].id).toBe("sf1");
  });

  it("marks response as not_found when no Japanese screenshots remain", () => {
    const parsed = parseVndbScreenshots({
      results: [
        {
          id: "v1",
          title: "Only EN",
          screenshots: [
            {
              id: "sf1",
              url: "https://example.com/en.jpg",
              thumbnail: "https://example.com/en-t.jpg",
              release: { languages: [{ lang: "en" }] },
            },
          ],
        },
      ],
    });

    expect(parsed.status).toBe("not_found");
    expect(parsed.screenshots).toEqual([]);
  });

  it("falls back to Japanese-included multilingual screenshots when no Japanese-only screenshots exist", () => {
    const parsed = parseVndbScreenshots({
      results: [
        {
          id: "v1",
          title: "KANADE",
          screenshots: [
            {
              id: "sf1",
              url: "https://example.com/multilingual.jpg",
              thumbnail: "https://example.com/multilingual-t.jpg",
              release: {
                languages: [
                  { lang: "en" },
                  { lang: "ja", mtl: false },
                  { lang: "zh-Hans" },
                ],
              },
            },
          ],
        },
      ],
    });

    expect(parsed.status).toBe("ok");
    expect(parsed.screenshots).toHaveLength(1);
    expect(parsed.screenshots[0].id).toBe("sf1");
  });

  it("still prefers Japanese-only screenshots over multilingual screenshots", () => {
    const parsed = parseVndbScreenshots({
      results: [
        {
          id: "v1",
          title: "KANADE",
          screenshots: [
            {
              id: "multilingual",
              url: "https://example.com/multilingual.jpg",
              thumbnail: "https://example.com/multilingual-t.jpg",
              release: {
                languages: [{ lang: "en" }, { lang: "ja", mtl: false }],
              },
            },
            {
              id: "ja-only",
              url: "https://example.com/ja.jpg",
              thumbnail: "https://example.com/ja-t.jpg",
              release: { languages: [{ lang: "ja", mtl: false }] },
            },
          ],
        },
      ],
    });

    expect(parsed.screenshots.map((s) => s.id)).toEqual(["ja-only"]);
  });

  it("uses title, brand and release date to choose the best VNDB candidate", () => {
    const parsed = parseVndbScreenshots(
      {
        results: [
          {
            id: "v-wrong",
            title: "日本語タイトル",
            alttitle: "日本語タイトル",
            released: "2022-01-01",
            developers: [{ name: "Other Brand", original: "別ブランド" }],
            screenshots: [
              {
                id: "wrong-shot",
                url: "https://example.com/wrong.jpg",
                thumbnail: "https://example.com/wrong-t.jpg",
                release: { languages: [{ lang: "ja" }] },
              },
            ],
          },
          {
            id: "v-correct",
            title: "Nihongo Title",
            alttitle: "日本語タイトル",
            titles: [
              {
                lang: "ja",
                title: "日本語タイトル",
                latin: "Nihongo Title",
                official: true,
                main: true,
              },
            ],
            aliases: ["日本語タイトル 完全版"],
            released: "2024-05-10",
            developers: [
              {
                name: "Test Brand",
                original: "テストブランド",
                aliases: ["てすとぶらんど"],
              },
            ],
            screenshots: [
              {
                id: "correct-shot",
                url: "https://example.com/correct.jpg",
                thumbnail: "https://example.com/correct-t.jpg",
                release: { languages: [{ lang: "ja" }] },
              },
            ],
          },
        ],
      },
      collectionElement(),
    );

    expect(parsed.vndbId).toBe("v-correct");
    expect(parsed.screenshots[0].id).toBe("correct-shot");
  });

  it("narrows by title first and then by release date before brand tie-breaks", () => {
    const parsed = parseVndbScreenshots(
      {
        results: [
          {
            id: "v-brand-only",
            title: "日本語タイトル",
            alttitle: "日本語タイトル",
            released: "2022-01-01",
            developers: [{ name: "Test Brand", original: "テストブランド" }],
            screenshots: [
              {
                id: "brand-shot",
                url: "https://example.com/brand.jpg",
                thumbnail: "https://example.com/brand-t.jpg",
                release: { languages: [{ lang: "ja" }] },
              },
            ],
          },
          {
            id: "v-date-match",
            title: "日本語タイトル",
            alttitle: "日本語タイトル",
            released: "2024-05-10",
            developers: [{ name: "Other Brand", original: "別ブランド" }],
            screenshots: [
              {
                id: "date-shot",
                url: "https://example.com/date.jpg",
                thumbnail: "https://example.com/date-t.jpg",
                release: { languages: [{ lang: "ja" }] },
              },
            ],
          },
        ],
      },
      collectionElement(),
    );

    expect(parsed.vndbId).toBe("v-date-match");
    expect(parsed.screenshots[0].id).toBe("date-shot");
  });

  it("matches KANADE to the Frontwing 2025 entry instead of older same-title entries", () => {
    const parsed = parseVndbScreenshots(
      {
        results: [
          {
            id: "v3309",
            title: "Kanade",
            alttitle: "奏 ～カナデ～",
            released: "2002-06-28",
            developers: [
              {
                name: "Purple software",
                original: "パープルソフトウェア",
              },
            ],
            screenshots: [
              {
                id: "old-shot",
                url: "https://example.com/old.jpg",
                thumbnail: "https://example.com/old-t.jpg",
                release: { languages: [{ lang: "ja", mtl: false }] },
              },
            ],
          },
          {
            id: "v52778",
            title: "KANADE",
            released: "2025-06-12",
            developers: [
              {
                name: "Frontwing Co., Ltd.",
                aliases: ["Front Wing", "フロントウイング"],
              },
            ],
            screenshots: [
              {
                id: "kanade-shot",
                url: "https://example.com/kanade.jpg",
                thumbnail: "https://example.com/kanade-t.jpg",
                release: {
                  languages: [
                    { lang: "en", mtl: false },
                    { lang: "ja", mtl: false },
                    { lang: "zh-Hans", mtl: false },
                  ],
                },
              },
            ],
          },
        ],
      },
      collectionElement({
        gamename: "KANADE",
        brandname: "フロントウイング",
        sellday: "2025-06-12",
      }),
    );

    expect(parsed.vndbId).toBe("v52778");
    expect(parsed.screenshots[0].id).toBe("kanade-shot");
  });

  it("filters sensitive screenshots unless explicitly enabled", () => {
    const source = [
      screenshot({ id: "safe", sexual: 0, violence: 0 }),
      screenshot({ id: "sexual", sexual: 1.5, violence: 0 }),
      screenshot({ id: "violent", sexual: 0, violence: 1.5 }),
    ];

    expect(filterVndbScreenshots(source, false).map((s) => s.id)).toEqual([
      "safe",
    ]);
    expect(filterVndbScreenshots(source, true)).toHaveLength(3);
  });

  it("treats ok and not_found caches as fresh for 30 days", () => {
    const cache: VndbScreenshotCache = {
      collectionElementId: 1,
      vndbId: "v1",
      matchedTitle: "title",
      screenshotsJson: JSON.stringify({ version: 3, screenshots: [] }),
      fetchedAt: "2026-05-01T00:00:00.000Z",
      status: "not_found",
    };

    expect(isFreshVndbCache(cache, new Date("2026-05-10T00:00:00.000Z"))).toBe(
      true,
    );
    expect(isFreshVndbCache({ ...cache, status: "error" })).toBe(false);
    expect(isFreshVndbCache({ ...cache, screenshotsJson: "[]" })).toBe(false);
  });

  it("reads screenshots from valid ok cache only", () => {
    const screenshotsJson = JSON.stringify({
      version: 3,
      screenshots: [screenshot()],
    });
    const cache: VndbScreenshotCache = {
      collectionElementId: 1,
      vndbId: "v1",
      matchedTitle: "title",
      screenshotsJson,
      fetchedAt: "2026-05-01T00:00:00.000Z",
      status: "ok",
    };

    expect(readScreenshotsFromCache(cache)).toHaveLength(1);
    expect(
      readScreenshotsFromCache({
        ...cache,
        screenshotsJson: JSON.stringify([screenshot()]),
      }),
    ).toHaveLength(1);
    expect(readScreenshotsFromCache({ ...cache, status: "not_found" })).toEqual(
      [],
    );
  });

  it("requests screenshot release language fields", () => {
    expect(createVndbRequestBody("test").fields).toContain(
      "release.languages{lang,mtl}",
    );
    expect(createVndbRequestBody("test").fields).toContain(
      "developers{name,original,aliases}",
    );
    expect(createVndbRequestBody("test").fields).toContain(
      "titles{lang,title,latin,official,main}",
    );
    expect(createVndbRequestBody("test").results).toBe(10);
  });
});
