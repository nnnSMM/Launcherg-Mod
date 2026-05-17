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
  parseDlsiteScreenshotsFromProductHtml,
  parseFanzaScreenshotsFromHtml,
  parseFanzaScreenshotsFromProductHtml,
  parseSteamScreenshotsFromProductHtml,
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
      screenshotsJson: JSON.stringify({ version: 10, screenshots: [] }),
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
