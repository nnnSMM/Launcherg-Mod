import { describe, expect, it, vi } from "vitest";

vi.mock("@/lib/command", () => ({
  commandGetAppSetting: vi.fn(),
  commandGetGameScreenshotCache: vi.fn(),
  commandUpsertGameScreenshotCache: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-http", () => ({
  fetch: vi.fn(),
}));

import {
  createApiRequestBody,
  ensureGameScreenshotCache,
  filterGameScreenshots,
  isFreshGameCache,
  parseDlsiteScreenshotsFromProductHtml,
  parseFanzaScreenshotsFromHtml,
  parseFanzaScreenshotsFromProductHtml,
  parseSteamScreenshotsFromProductHtml,
  parseGameScreenshots,
  readScreenshotsFromCache,
} from "./useGameScreenshots";
import { fetch as tauriHttpFetch } from "../mock/tauri-http";
import { fetch as tauriHttpPluginFetch } from "@tauri-apps/plugin-http";
import type { GameScreenshot, GameScreenshotCache } from "./types";
import type { CollectionElement } from "./types";

const screenshot = (overrides: Partial<GameScreenshot> = {}): GameScreenshot => ({
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
  firstPlayAt: null,
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

describe("useGameScreenshots", () => {
  it("extracts FANZA sample CG images from ErogameScape HTML", () => {
    const parsed = parseFanzaScreenshotsFromHtml(`
      <div id="game_title"><a>FANZA Sample Title</a></div>
      <div id="dmm_sample_cg_main">
        <a><img src="https://pics.dmm.co.jp/digital/pcgame/test_0001/test_0001js-001.jpg"></a>
        <a><img src="https://pics.dmm.co.jp/digital/pcgame/test_0001/test_0001js-002.jpg"></a>
        <a><img src="https://pics.dmm.co.jp/digital/pcgame/test_0001/test_0001js-001.jpg"></a>
      </div>
      <div id="left_dmm_img">
        <img src="https://pics.dmm.co.jp/digital/pcgame/test_0001/test_0001pl.jpg">
      </div>
    `);

    expect(parsed.matchedTitle).toBe("FANZA Sample Title");
    expect(parsed.productPageUrl).toBe(null);
    expect(parsed.dlsiteProductPageUrl).toBe(null);
    expect(parsed.steamProductPageUrl).toBe(null);
    expect(parsed.screenshots.map((s) => s.thumbnail)).toEqual([
      "https://pics.dmm.co.jp/digital/pcgame/test_0001/test_0001js-001.jpg",
      "https://pics.dmm.co.jp/digital/pcgame/test_0001/test_0001js-002.jpg",
    ]);
    expect(parsed.screenshots.map((s) => s.url)).toEqual([
      "https://pics.dmm.co.jp/digital/pcgame/test_0001/test_0001jp-001.jpg",
      "https://pics.dmm.co.jp/digital/pcgame/test_0001/test_0001jp-002.jpg",
    ]);
  });

  it("falls back to the left FANZA image when the sample CG section is absent", () => {
    const parsed = parseFanzaScreenshotsFromHtml(`
      <div id="game_title"><a>花束を君に贈ろう -Kinsenka-</a></div>
      <a href="https://al.fanza.co.jp/?lurl=https%3A%2F%2Fdlsoft.dmm.co.jp%2Fdetail%2Ffwing_0045%2F&af_id=egsa-001">
        DMM
      </a>
      <div id="main_image">
        <img src="https://pics.dmm.co.jp/digital/pcgame/fwing_0045/fwing_0045pl.jpg">
      </div>
      <div id="images_aff_sites">
        <div id="left_dlsite_img">
          <img src="//img.dlsite.jp/modpub/images2/work/professional/VJ01005000/VJ01004357_img_smpa2.jpg">
        </div>
        <div id="left_dmm_img">
          <img src="https://pics.dmm.co.jp/digital/pcgame/fwing_0045/fwing_0045js-005.jpg">
        </div>
      </div>
    `);

    expect(parsed.matchedTitle).toBe("花束を君に贈ろう -Kinsenka-");
    expect(parsed.productPageUrl).toBe("https://dlsoft.dmm.com/detail/fwing_0045/");
    expect(parsed.screenshots.map((s) => s.thumbnail)).toEqual([
      "https://pics.dmm.co.jp/digital/pcgame/fwing_0045/fwing_0045js-005.jpg",
    ]);
    expect(parsed.screenshots.map((s) => s.url)).toEqual([
      "https://pics.dmm.co.jp/digital/pcgame/fwing_0045/fwing_0045jp-005.jpg",
    ]);
  });

  it("falls back to DLsite sample images when FANZA images are absent", () => {
    const parsed = parseFanzaScreenshotsFromHtml(`
      <div id="game_title"><a>プトリカ 1st.cut:The Reason She Must Perish</a></div>
      <a href="https://www.dlsite.com/home/dlaf/=/link/work/aid/erogamescape/id/RJ01318457.html">
        DLsite.com
      </a>
      <div id="dlsite_sample_cg_main">
        <img src="//img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp1.jpg">
        <img src="//img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp2.jpg">
      </div>
      <div id="left_dlsite_img">
        <img src="//img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp1.jpg">
      </div>
    `);

    expect(parsed.matchedTitle).toBe("プトリカ 1st.cut:The Reason She Must Perish");
    expect(parsed.productPageUrl).toBe(null);
    expect(parsed.dlsiteProductPageUrl).toBe(
      "https://www.dlsite.com/home/work/=/product_id/RJ01318457.html",
    );
    expect(parsed.steamProductPageUrl).toBe(null);
    expect(parsed.screenshots.map((s) => s.url)).toEqual([
      "https://img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp1.jpg",
      "https://img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp2.jpg",
    ]);
  });

  it("extracts a Steam product page URL when no shop sample source is present", () => {
    const parsed = parseFanzaScreenshotsFromHtml(`
      <div id="game_title"><a>魔法少女ノ魔女裁判</a></div>
      <div id="bottom_inter_links">
        <ul>
          <li><a href="https://manosaba.com/" target="_blank">game_OHP</a></li>
          <li><a href="https://store.steampowered.com/app/3101040/">STEAM</a></li>
          <li><a href="https://store.steampowered.com/app/3101040/_/" target="_blank">体験版</a>(Hなし)</li>
        </ul>
      </div>
    `);

    expect(parsed.matchedTitle).toBe("魔法少女ノ魔女裁判");
    expect(parsed.screenshots).toEqual([]);
    expect(parsed.productPageUrl).toBe(null);
    expect(parsed.dlsiteProductPageUrl).toBe(null);
    expect(parsed.steamProductPageUrl).toBe(
      "https://store.steampowered.com/app/3101040/",
    );
  });

  it("extracts Steam screenshots and pairs thumbnails with full images", () => {
    const screenshots = parseSteamScreenshotsFromProductHtml(`
      <img src="https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/2818450/ss_aaaaaaaa.116x65.jpg?t=1">
      <img src="https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/2818450/ss_aaaaaaaa.600x338.jpg?t=1">
      <img src="https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/2818450/ss_aaaaaaaa.1920x1080.jpg?t=1">
      <img src="https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/2818450/ss_bbbbbbbb.600x338.jpg?t=1">
    `);

    expect(screenshots).toEqual([
      {
        id: "steam-1-ss_aaaaaaaa",
        url: "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/2818450/ss_aaaaaaaa.1920x1080.jpg?t=1",
        thumbnail:
          "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/2818450/ss_aaaaaaaa.600x338.jpg?t=1",
        dims: null,
        thumbnailDims: null,
        sexual: 0,
        violence: 0,
        languages: ["ja"],
      },
      {
        id: "steam-2-ss_bbbbbbbb",
        url: "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/2818450/ss_bbbbbbbb.600x338.jpg?t=1",
        thumbnail:
          "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/2818450/ss_bbbbbbbb.600x338.jpg?t=1",
        dims: null,
        thumbnailDims: null,
        sexual: 0,
        violence: 0,
        languages: ["ja"],
      },
    ]);
  });

  it("extracts Steam screenshots from escaped store JSON", () => {
    const screenshots = parseSteamScreenshotsFromProductHtml(`
      &quot;screenshots&quot;:[{&quot;thumbnail&quot;:&quot;https:\\/\\/shared.fastly.steamstatic.com\\/store_item_assets\\/steam\\/apps\\/3101040\\/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53\\/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.116x65.jpg?t=1766600472&quot;,&quot;standard&quot;:&quot;https:\\/\\/shared.fastly.steamstatic.com\\/store_item_assets\\/steam\\/apps\\/3101040\\/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53\\/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.600x338.jpg?t=1766600472&quot;,&quot;full&quot;:&quot;https:\\/\\/shared.fastly.steamstatic.com\\/store_item_assets\\/steam\\/apps\\/3101040\\/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53\\/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.1920x1080.jpg?t=1766600472&quot;}]
    `);

    expect(screenshots).toHaveLength(1);
    expect(screenshots[0].thumbnail).toBe(
      "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/3101040/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.600x338.jpg?t=1766600472",
    );
    expect(screenshots[0].url).toBe(
      "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/3101040/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.1920x1080.jpg?t=1766600472",
    );
  });

  it("falls back to generic product images when a Steam page has no screenshot payload", () => {
    const screenshots = parseSteamScreenshotsFromProductHtml(`
      <html>
        <head>
          <meta property="og:image" content="https://example.com/cover.jpg">
          <meta name="twitter:image" content="https://example.com/preview.png">
        </head>
        <body>
          <img src="https://example.com/preview.png">
          <img src="https://example.com/logo.png">
        </body>
      </html>
    `);

    expect(screenshots).toEqual([
      {
        id: "steam-fallback-1-https://example.com/cover.jpg",
        url: "https://example.com/cover.jpg",
        thumbnail: "https://example.com/cover.jpg",
        dims: null,
        thumbnailDims: null,
        sexual: 0,
        violence: 0,
        languages: ["ja"],
      },
      {
        id: "steam-fallback-2-https://example.com/preview.png",
        url: "https://example.com/preview.png",
        thumbnail: "https://example.com/preview.png",
        dims: null,
        thumbnailDims: null,
        sexual: 0,
        violence: 0,
        languages: ["ja"],
      },
    ]);
  });

  it("normalizes DLsite resized thumbnails to full sample images", () => {
    const screenshots = parseDlsiteScreenshotsFromProductHtml(`
      <img src="//img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_main.jpg">
      <img src="//img.dlsite.jp/resize/images2/work/doujin/RJ01319000/RJ01318457_img_smp1_100x100.jpg">
      <img src="//img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp1.jpg">
      <img src="//img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp2.jpg">
    `);

    expect(screenshots.map((s) => s.thumbnail)).toEqual([
      "https://img.dlsite.jp/resize/images2/work/doujin/RJ01319000/RJ01318457_img_smp1_100x100.jpg",
      "https://img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp2.jpg",
    ]);
    expect(screenshots.map((s) => s.url)).toEqual([
      "https://img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp1.jpg",
      "https://img.dlsite.jp/modpub/images2/work/doujin/RJ01319000/RJ01318457_img_smp2.jpg",
    ]);
  });

  it("extracts all sample images from a DMM product page", () => {
    const screenshots = parseFanzaScreenshotsFromProductHtml(`
      <img src="https://pics.dmm.com/digital/pcgame/fwing_0045/fwing_0045pl.jpg">
      <img src="https://pics.dmm.com/digital/pcgame/fwing_0045/fwing_0045jp-001.jpg">
      <img src="https://pics.dmm.com/digital/pcgame/fwing_0045/fwing_0045js-001.jpg">
      <img src="https://pics.dmm.com/digital/pcgame/fwing_0045/fwing_0045js-002.jpg">
      <img src="https://pics.dmm.com/digital/pcgame/guide/fwing_0045/cha001.jpg">
    `);

    expect(screenshots.map((s) => s.thumbnail)).toEqual([
      "https://pics.dmm.com/digital/pcgame/fwing_0045/fwing_0045js-001.jpg",
      "https://pics.dmm.com/digital/pcgame/fwing_0045/fwing_0045js-002.jpg",
    ]);
    expect(screenshots.map((s) => s.url)).toEqual([
      "https://pics.dmm.com/digital/pcgame/fwing_0045/fwing_0045jp-001.jpg",
      "https://pics.dmm.com/digital/pcgame/fwing_0045/fwing_0045jp-002.jpg",
    ]);
  });

  it("extracts only Japanese-release screenshots from GAME response", () => {
    const parsed = parseGameScreenshots({
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
    expect(parsed.matchedTitle).toBe("日本語タイトル");
    expect(parsed.screenshots).toHaveLength(1);
    expect(parsed.screenshots[0].id).toBe("sf1");
  });

  it("marks response as not_found when no Japanese screenshots remain", () => {
    const parsed = parseGameScreenshots({
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
    const parsed = parseGameScreenshots({
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
    const parsed = parseGameScreenshots({
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

  it("uses title, brand and release date to choose the best GAME candidate", () => {
    const parsed = parseGameScreenshots(
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

    expect(parsed.screenshots[0].id).toBe("correct-shot");
  });

  it("narrows by title first and then by release date before brand tie-breaks", () => {
    const parsed = parseGameScreenshots(
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

    expect(parsed.screenshots[0].id).toBe("date-shot");
  });

  it("matches KANADE to the Frontwing 2025 entry instead of older same-title entries", () => {
    const parsed = parseGameScreenshots(
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

    expect(parsed.screenshots[0].id).toBe("kanade-shot");
  });

  it("filters sensitive screenshots unless explicitly enabled", () => {
    const source = [
      screenshot({ id: "safe", sexual: 0, violence: 0 }),
      screenshot({ id: "sexual", sexual: 1.5, violence: 0 }),
      screenshot({ id: "violent", sexual: 0, violence: 1.5 }),
    ];

    expect(filterGameScreenshots(source, false).map((s) => s.id)).toEqual([
      "safe",
    ]);
    expect(filterGameScreenshots(source, true)).toHaveLength(3);
  });

  it("treats ok and not_found caches as fresh for 30 days", () => {
    const cache: GameScreenshotCache = {
      collectionElementId: 1,
      matchedTitle: "title",
      screenshotsJson: JSON.stringify({ version: 14, screenshots: [] }),
      fetchedAt: "2026-05-01T00:00:00.000Z",
      status: "not_found",
    };

    expect(isFreshGameCache(cache, new Date("2026-05-10T00:00:00.000Z"))).toBe(
      true,
    );
    expect(isFreshGameCache({ ...cache, status: "error" })).toBe(false);
    expect(isFreshGameCache({ ...cache, screenshotsJson: "[]" })).toBe(false);
  });

  it("reads screenshots from valid ok cache only", () => {
    const screenshotsJson = JSON.stringify({
      version: 3,
      screenshots: [screenshot()],
    });
    const cache: GameScreenshotCache = {
      collectionElementId: 1,
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
    expect(createApiRequestBody("test").fields).toContain(
      "release.languages{lang,mtl}",
    );
    expect(createApiRequestBody("test").fields).toContain(
      "developers{name,original,aliases}",
    );
    expect(createApiRequestBody("test").fields).toContain(
      "titles{lang,title,latin,official,main}",
    );
    expect(createApiRequestBody("test").results).toBe(10);
  });

  it("extracts DMM mono/pcgame product page URL from affiliate link in lurl parameter", () => {
    const parsed = parseFanzaScreenshotsFromHtml(`
      <a href="https://al.fanza.co.jp/?lurl=https%3A%2F%2Fwww.dmm.co.jp%2Fmono%2Fpcgame%2F-%2Fdetail%2F%3D%2Fcid%3D774apc13854%2F&af_id=egsa-001">DMM</a>
    `);
    expect(parsed.productPageUrl).toBe("https://www.dmm.co.jp/mono/pcgame/-/detail/=/cid=774apc13854/");
  });

  it("extracts screenshots from DMM mono/pcgame product page", () => {
    const screenshots = parseFanzaScreenshotsFromProductHtml(`
      <div id="sample-image-block">
        <a href="https://pics.dmm.co.jp/mono/game/774apc13854/774apc13854-1.jpg">
          <img src="https://pics.dmm.co.jp/mono/game/774apc13854/774apc13854-1.jpg">
        </a>
        <a href="https://pics.dmm.co.jp/mono/game/774apc13854/774apc13854-2.jpg">
          <img src="https://pics.dmm.co.jp/mono/game/774apc13854/774apc13854-2.jpg">
        </a>
      </div>
    `);
    expect(screenshots).toHaveLength(2);
    expect(screenshots[0].url).toBe("https://pics.dmm.co.jp/mono/game/774apc13854/774apc13854-1.jpg");
    expect(screenshots[0].thumbnail).toBe("https://pics.dmm.co.jp/mono/game/774apc13854/774apc13854-1.jpg");
  });

  it("extracts screenshots from DLsite professional product page using data-src", () => {
    const screenshots = parseDlsiteScreenshotsFromProductHtml(`
      <div class="product-slider-data">
        <div data-src="//img.dlsite.jp/modpub/images2/work/professional/VJ015000/VJ014408_img_main.jpg" data-thumb="//img.dlsite.jp/resize/images2/work/professional/VJ015000/VJ014408_img_main_240x240.jpg"></div>
        <div data-src="//img.dlsite.jp/modpub/images2/work/professional/VJ015000/VJ014408_img_smpa1.jpg" data-thumb="//img.dlsite.jp/resize/images2/work/professional/VJ015000/VJ014408_img_smpa1_100x100.jpg"></div>
        <div data-src="//img.dlsite.jp/modpub/images2/work/professional/VJ015000/VJ014408_img_smpa2.jpg" data-thumb="//img.dlsite.jp/resize/images2/work/professional/VJ015000/VJ014408_img_smpa2_100x100.jpg"></div>
      </div>
    `);
    expect(screenshots).toHaveLength(2);
    expect(screenshots[0].url).toBe("https://img.dlsite.jp/modpub/images2/work/professional/VJ015000/VJ014408_img_smpa1.jpg");
    expect(screenshots[0].thumbnail).toBe("https://img.dlsite.jp/resize/images2/work/professional/VJ015000/VJ014408_img_smpa1_100x100.jpg");
  });

  // 9-nine-(ID 29958)の実際のケース:
  // ErogameScapeページにはDMMアフィリエイトリンクとDLsiteサムネイルの両方が存在する。
  // DMMページは地域制限で画像が取得できないため、DLsiteサムネイル（ErogameScapeに埋め込み済み）を
  // 利用するか、DLsite製品ページからフル解像度画像を取得するべきである。
  it("uses DLsite product page screenshots when DMM productPageUrl yields no images (9-nine- scenario)", async () => {
    vi.mocked(tauriHttpPluginFetch).mockImplementation(tauriHttpFetch as any);

    // 実際の29958ページに近い構造:
    // DMMアフィリエイトリンクあり、DLsite製品リンクあり、DLsiteサムネイルあり
    const mockErogamescapeHtml = `
      <h2 id="game_title"><a href="https://9-nine-project.com/">9-nine-</a></h2>
      <div id="main_image">
        <a href="https://al.fanza.co.jp/?lurl=https%3A%2F%2Fwww.dmm.co.jp%2Fmono%2Fpcgame%2F-%2Fdetail%2F%3D%2Fcid%3D774apc13854%2F&af_id=egsa-001">
          <img src="https://pics.dmm.co.jp/mono/game/774apc13854/774apc13854pl.jpg">
        </a>
      </div>
      <div id="dlsite_sample_cg_1">
        <div id="dlsite_sample_cg_1_main">
          <a href="https://www.dlsite.com/soft/dlaf/=/link/work/aid/erogamescape/id/VJ014408.html">
            <img src="//img.dlsite.jp/resize/images2/work/professional/VJ015000/VJ014408_img_smpa1_100x100.jpg">
          </a>
          <a href="https://www.dlsite.com/soft/dlaf/=/link/work/aid/erogamescape/id/VJ014408.html">
            <img src="//img.dlsite.jp/resize/images2/work/professional/VJ015000/VJ014408_img_smpa2_100x100.jpg">
          </a>
        </div>
      </div>
    `;

    // DMMページは地域制限エラー（画像なし）
    const mockDmmRegionErrorHtml = `
      <html><body><div class="foreignError">Sorry! This content is not available in your region.</div></body></html>
    `;

    // DLsite製品ページはJavaScript動的なので静的HTMLには画像URLなし（DLsite本番相当）
    const mockDlsiteProductHtml = `
      <html><body><div class="work_parts_area"><!-- JS rendered slider --></div></body></html>
    `;

    const globalFetch = vi.spyOn(window, "fetch").mockImplementation(async (url) => {
      const urlStr = String(url);
      if (urlStr.includes("game.php?game=29958") || urlStr.includes("29958.html")) {
        return { ok: true, text: async () => mockErogamescapeHtml } as any;
      }
      if (urlStr.includes("cid=774apc13854") || urlStr.includes("774apc13854")) {
        return { ok: true, text: async () => mockDmmRegionErrorHtml } as any;
      }
      if (urlStr.includes("VJ014408")) {
        return { ok: true, text: async () => mockDlsiteProductHtml } as any;
      }
      return { ok: false } as any;
    });

    const { commandGetGameScreenshotCache } = await import("@/lib/command");
    vi.mocked(commandGetGameScreenshotCache).mockResolvedValue(null);

    const element = collectionElement({ id: 29958, gamename: "9-nine-" });
    const cache = await ensureGameScreenshotCache(element);

    // DMMで画像が取れなくても、ErogameScapeに埋め込まれたDLsiteサムネイルが利用されること
    expect(cache.status).toBe("ok");
    const screenshots = JSON.parse(cache.screenshotsJson).screenshots;
    expect(screenshots.length).toBeGreaterThan(0);
    // ErogameScapeページのDLsiteサムネイルがフルURLに変換されること
    expect(screenshots[0].url).toBe(
      "https://img.dlsite.jp/modpub/images2/work/professional/VJ015000/VJ014408_img_smpa1.jpg",
    );

    globalFetch.mockRestore();
  });

  describe("tauri-http mock fetch integration", () => {
    it("fetches DLsite mock HTML and extracts screenshots", async () => {
      vi.mocked(tauriHttpPluginFetch).mockImplementation(tauriHttpFetch as any);

      const mockErogamescapeHtml = `
        <div id="game_title"><a>終のステラ</a></div>
        <div id="images_aff_sites">
          <div id="left_dlsite_img">
            <a href="https://www.dlsite.com/home/work/=/product_id/VJ015604.html">DLsite</a>
          </div>
        </div>
      `;

      const mockDlsiteHtml = `
        <div id="dlsite_sample_cg_main">
          <img src="//img.dlsite.jp/resize/images2/work/professional/VJ016000/VJ015604_img_smpa1_200x200.jpg">
        </div>
      `;

      const globalFetch = vi.spyOn(window, "fetch").mockImplementation(async (url) => {
        const urlStr = String(url);
        if (urlStr.includes("38696.html") || urlStr.includes("game.php?game=38696")) {
          return { ok: true, text: async () => mockErogamescapeHtml } as any;
        }
        if (urlStr.includes("dlsite-VJ015604.html") || urlStr.includes("VJ015604")) {
          return { ok: true, text: async () => mockDlsiteHtml } as any;
        }
        return { ok: false } as any;
      });

      const { commandGetGameScreenshotCache } = await import("@/lib/command");
      vi.mocked(commandGetGameScreenshotCache).mockResolvedValue(null);

      const element = collectionElement({ id: 38696, gamename: "終のステラ" });
      const cache = await ensureGameScreenshotCache(element);

      expect(cache.status).toBe("ok");
      expect(cache.matchedTitle).toBe("終のステラ");
      const screenshots = JSON.parse(cache.screenshotsJson).screenshots;
      expect(screenshots).toHaveLength(1);
      expect(screenshots[0].url).toBe("https://img.dlsite.jp/modpub/images2/work/professional/VJ016000/VJ015604_img_smpa1.jpg");

      globalFetch.mockRestore();
    });

    it("fetches Steam mock HTML and extracts screenshots", async () => {
      vi.mocked(tauriHttpPluginFetch).mockImplementation(tauriHttpFetch as any);

      const mockErogamescapeHtml = `
        <div id="game_title"><a>魔法少女ノ魔女裁判</a></div>
        <div id="bottom_inter_links">
          <ul>
            <li><a href="https://store.steampowered.com/app/3101040/">STEAM</a></li>
          </ul>
        </div>
      `;

      const mockSteamHtml = `
        &quot;screenshots&quot;:[{&quot;thumbnail&quot;:&quot;https:\\/\\/shared.fastly.steamstatic.com\\/store_item_assets\\/steam\\/apps\\/3101040\\/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53\\/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.116x65.jpg?t=1766600472&quot;,&quot;standard&quot;:&quot;https:\\/\\/shared.fastly.steamstatic.com\\/store_item_assets\\/steam\\/apps\\/3101040\\/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53\\/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.600x338.jpg?t=1766600472&quot;,&quot;full&quot;:&quot;https:\\/\\/shared.fastly.steamstatic.com\\/store_item_assets\\/steam\\/apps\\/3101040\\/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53\\/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.1920x1080.jpg?t=1766600472&quot;}]
      `;

      const globalFetch = vi.spyOn(window, "fetch").mockImplementation(async (url) => {
        const urlStr = String(url);
        if (urlStr.includes("38631.html") || urlStr.includes("game.php?game=38631")) {
          return { ok: true, text: async () => mockErogamescapeHtml } as any;
        }
        if (urlStr.includes("steam-3101040.html") || urlStr.includes("steampowered.com/app/3101040")) {
          return { ok: true, text: async () => mockSteamHtml } as any;
        }
        return { ok: false } as any;
      });

      const { commandGetGameScreenshotCache } = await import("@/lib/command");
      vi.mocked(commandGetGameScreenshotCache).mockResolvedValue(null);

      const element = collectionElement({ id: 38631, gamename: "魔法少女ノ魔女裁判" });
      const cache = await ensureGameScreenshotCache(element);

      expect(cache.status).toBe("ok");
      const screenshots = JSON.parse(cache.screenshotsJson).screenshots;
      expect(screenshots).toHaveLength(1);
      expect(screenshots[0].url).toBe("https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/3101040/6b66a3dc43a2f99d365ce94dc0a28ff23e742b53/ss_6b66a3dc43a2f99d365ce94dc0a28ff23e742b53.1920x1080.jpg?t=1766600472");

      globalFetch.mockRestore();
    });

    it("extracts only permitted DLsite screenshots for Golden Loveriche (25861)", async () => {
      vi.mocked(tauriHttpPluginFetch).mockImplementation(tauriHttpFetch as any);

      const fs = await import("node:fs");
      const path = await import("node:path");
      const htmlPath = path.resolve(process.cwd(), "public/demo-data/25861.html");
      const erogamescapeHtml = fs.readFileSync(htmlPath, "utf-8");

      const globalFetch = vi.spyOn(window, "fetch").mockImplementation(async (url) => {
        const urlStr = String(url);
        if (urlStr.includes("25861.html") || urlStr.includes("game.php?game=25861")) {
          return { ok: true, text: async () => erogamescapeHtml } as any;
        }
        return { ok: false } as any;
      });

      const { commandGetGameScreenshotCache } = await import("@/lib/command");
      vi.mocked(commandGetGameScreenshotCache).mockResolvedValue(null);

      const element = collectionElement({ id: 25861, gamename: "金色ラブリッチェ" });
      const cache = await ensureGameScreenshotCache(element);

      expect(cache.status).toBe("ok");
      const screenshots = JSON.parse(cache.screenshotsJson).screenshots;

      expect(screenshots).toHaveLength(8);

      const urls = screenshots.map((s: any) => s.url);
      for (const url of urls) {
        expect(url).toMatch(/VJ011419_img_smpa\d+\.jpg/);
      }

      globalFetch.mockRestore();
    });
  });
});
