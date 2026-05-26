import { afterEach, describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/plugin-http", () => ({
  fetch: vi.fn(),
}));

import {
  createDlsiteAnnounceUrl,
  createDlsiteDetailUrl,
  createFanzaDetailUrl,
  createFanzaDoujinDetailUrl,
  createSteamAppDetailsUrl,
  extractDlsiteId,
  extractDlsiteProductIds,
  extractExternalIdsFromDocument,
  extractFanzaId,
  extractSteamAppId,
  extractStorySectionHtml,
  fetchFromDlsite,
  findSteamAppIdFromOfficialDocument,
  getWorkByScrape,
  getCreator,
  getDlsiteDescriptionFromDocument,
  getDlsiteDescriptionHtmlFromDocument,
  getFanzaDescriptionFromDocument,
  getMusics,
  getSteamDescriptionFromAppDetails,
  getVoiceActors,
  isTextLikelyRelated,
} from "./scrapeWork";
import { fetch as tauriHttpFetch } from "../mock/tauri-http";
import { fetch as tauriHttpPluginFetch } from "@tauri-apps/plugin-http";
import { getWorkById } from "@/mock/demoCatalog";

const jpTitle = "\u9b54\u6cd5\u5c11\u5973";
const fanzaStory =
  "\u65e5\u672c\u8a9e\u306eFANZA\u30b9\u30c8\u30fc\u30ea\u30fc\u672c\u6587\u3067\u3059\u3002";
const dlsiteStory =
  "\u65e5\u672c\u8a9e\u306eDLsite\u30b9\u30c8\u30fc\u30ea\u30fc\u672c\u6587\u3067\u3059\u3002";
const steamStory =
  "\u65e5\u672c\u8a9e\u306eSteam\u30b9\u30c8\u30fc\u30ea\u30fc\u672c\u6587\u3067\u3059\u3002";
const steamAbout =
  "\u65e5\u672c\u8a9e\u306eSteam\u6982\u8981\u672c\u6587\u3067\u3059\u3002";
const steamShort =
  "\u65e5\u672c\u8a9e\u306eSteam\u77ed\u3044\u8aac\u660e\u3067\u3059\u3002";

afterEach(() => {
  vi.mocked(tauriHttpPluginFetch).mockReset();
  vi.restoreAllMocks();
});

describe("scrapeWork", () => {
  describe("getCreator", () => {
    it("should extract creators from anchor tags", () => {
      const container = document.createElement("div");
      container.innerHTML = `
        <a href="creater.php?creater=101">Creator A</a>
        <a href="creater.php?creater=202">Creator B</a>
      `;
      const creators = getCreator(container);
      expect(creators).toHaveLength(2);
      expect(creators[0]).toEqual({ id: 101, name: "Creator A" });
      expect(creators[1]).toEqual({ id: 202, name: "Creator B" });
    });

    it("should handle links without ID gracefully", () => {
      const container = document.createElement("div");
      container.innerHTML = `<a href="other.php">Unknown</a>`;
      const creators = getCreator(container);
      expect(creators).toHaveLength(1);
      expect(creators[0].id).toBe(0);
      expect(creators[0].name).toBe("Unknown");
    });
  });

  describe("getVoiceActors", () => {
    it("should extract voice actors with roles and importance", () => {
      const container = document.createElement("div");
      container.innerHTML = `
        <a href="creater.php?creater=10">Actor A</a>
        <span style="color:black">Role A</span>
        <a href="creater.php?creater=20">Actor B</a>
        <span style="font-weight:bold">Role B</span>
        <a href="creater.php?creater=30">Actor C</a>
        <span style="color:gray">Role C</span>
      `;
      const actors = getVoiceActors(container);

      expect(actors).toHaveLength(3);
      expect(actors[0].id).toBe(10);
      expect(actors[0].name).toBe("Actor A");
      expect(actors[0].role).toBe("Role A");
      expect(actors[0].importance).toBe(1);
      expect(actors[1].id).toBe(20);
      expect(actors[1].name).toBe("Actor B");
      expect(actors[1].role).toBe("Role B");
      expect(actors[1].importance).toBe(0);
      expect(actors[2].id).toBe(30);
      expect(actors[2].name).toBe("Actor C");
      expect(actors[2].role).toBe("Role C");
      expect(actors[2].importance).toBe(2);
    });
  });

  describe("getMusics", () => {
    it("should extract music titles from table cells", () => {
      const row = document.createElement("tr");
      const cell1 = document.createElement("td");
      cell1.innerHTML = `<a href="music.php?music=1">Music 1</a>`;
      const cell2 = document.createElement("td");
      cell2.innerHTML = `<a href="music.php?music=2">Music 2</a>`;
      const cell3 = document.createElement("td");
      cell3.innerHTML = `Just Text`;

      row.appendChild(cell1);
      row.appendChild(cell2);
      row.appendChild(cell3);

      const musics = getMusics(row.getElementsByTagName("td"));

      expect(musics).toHaveLength(2);
      expect(musics[0]).toBe("Music 1");
      expect(musics[1]).toBe("Music 2");
    });
  });

  describe("Japanese story description sources", () => {
    it("extracts provider IDs only from ErogeScape URLs and text", () => {
      expect(extractFanzaId("https://dlsoft.dmm.co.jp/detail/abc_0001/")).toBe(
        "abc_0001",
      );
      expect(
        extractFanzaId(
          "https://www.dmm.co.jp/mono/pcgame/-/detail/=/cid=774apc13854/",
        ),
      ).toBe("774apc13854");
      expect(extractDlsiteId("https://www.dlsite.com/pro/work/=/product_id/VJ000123.html")).toBe(
        "VJ000123",
      );
      expect(extractSteamAppId("https://store.steampowered.com/app/420110/")).toBe(
        "420110",
      );
    });

    it("matches FANZA titles with award suffixes and fullwidth minus signs", () => {
      expect(
        isTextLikelyRelated(
          "\u30b5\u30af\u30e9\u30ce\u8a69 \u2212\u6afb\u306e\u68ee\u306e\u4e0a\u3092\u821e\u3046\u2212\u3010\u840c\u3048\u30b2\u30fc\u30a2\u30ef\u30fc\u30c92015 \u5927\u8cde\u3011",
          "\u30b5\u30af\u30e9\u30ce\u8a69 -\u6afb\u306e\u68ee\u306e\u4e0a\u3092\u821e\u3046-",
        ),
      ).toBe(true);
    });

    it("extracts external IDs from ErogeScape affiliate links without name search", () => {
      const fanzaUrl = encodeURIComponent(
        "https://dlsoft.dmm.co.jp/detail/fanza_0001/",
      );
      const doc = new DOMParser().parseFromString(
        `
          <div id="main">
            <a href="https://al.fanza.co.jp/?lurl=${fanzaUrl}&af_id=egsa-001">DMM</a>
            <a href="https://www.dlsite.com/soft/dlaf/=/link/work/aid/erogamescape/id/VJ015604.html">DLsite</a>
            <a href="https://store.steampowered.com/app/3101040/">STEAM</a>
          </div>
        `,
        "text/html",
      );

      expect(extractExternalIdsFromDocument(doc)).toEqual({
        fanzaId: "fanza_0001",
        dlsiteId: "VJ015604",
        steamAppId: "3101040",
      });
    });

    it("ignores ErogeScape toolbar ad links when extracting external IDs", () => {
      const toolbarUrl = encodeURIComponent(
        "https://dlsoft.dmm.co.jp/detail/unrelated_0001/",
      );
      const doc = new DOMParser().parseFromString(
        `
          <div id="main">
            <a href="https://al.fanza.co.jp/?lurl=${toolbarUrl}&af_id=egsa-001&ch=toolbar&ch_id=link">DMM</a>
          </div>
        `,
        "text/html",
      );

      expect(extractExternalIdsFromDocument(doc)).toEqual({});
    });

    it("extracts a Steam app ID from an official site document", () => {
      const doc = new DOMParser().parseFromString(
        `
          <html>
            <body>
              <a href="https://store.steampowered.com/app/3290440/">Steam Store</a>
            </body>
          </html>
        `,
        "text/html",
      );

      expect(findSteamAppIdFromOfficialDocument(doc)).toBe("3290440");
    });

    it("builds direct detail URLs from confirmed IDs", () => {
      expect(createFanzaDetailUrl("abc_0001")).toBe(
        "https://dlsoft.dmm.co.jp/detail/abc_0001/",
      );
      expect(createFanzaDoujinDetailUrl("d_123456")).toBe(
        "https://www.dmm.co.jp/dc/doujin/-/detail/=/cid=d_123456/",
      );
      expect(
        extractDlsiteProductIds(
          "folder RJ123456 and vj000123 and RE9999 and RJ123456",
        ),
      ).toEqual(["RJ123456", "VJ000123", "RE9999"]);
      expect(createDlsiteDetailUrl("VJ000123")).toBe(
        "https://www.dlsite.com/pro/work/=/product_id/VJ000123.html?locale=ja",
      );
      expect(createDlsiteDetailUrl("rj123456")).toBe(
        "https://www.dlsite.com/maniax/work/=/product_id/RJ123456.html?locale=ja",
      );
      expect(createDlsiteDetailUrl("RJ123456", "")).toBe(
        "https://www.dlsite.com/maniax/work/=/product_id/RJ123456.html?locale=ja",
      );
      expect(createDlsiteDetailUrl("RJ123456", "home")).toBe(
        "https://www.dlsite.com/home/work/=/product_id/RJ123456.html?locale=ja",
      );
      expect(createDlsiteAnnounceUrl("VJ013711", "pro")).toBe(
        "https://www.dlsite.com/pro/announce/=/product_id/VJ013711.html?locale=ja",
      );

      const appDetailsUrl = new URL(createSteamAppDetailsUrl(3101040));
      expect(appDetailsUrl.searchParams.get("appids")).toBe("3101040");
      expect(appDetailsUrl.searchParams.get("l")).toBe("japanese");
      expect(appDetailsUrl.searchParams.get("cc")).toBe("JP");
    });

    it("extracts story blocks from a heading until the next non-story section", () => {
      const html = `
        <p>Ignored lead text</p>
        <h2>\u30b9\u30c8\u30fc\u30ea\u30fc</h2>
        <p>${fanzaStory}</p>
        <h2>\u30ad\u30e3\u30e9\u30af\u30bf\u30fc</h2>
        <p>Excluded character text.</p>
      `;

      expect(extractStorySectionHtml(html)).toContain(fanzaStory);
      expect(extractStorySectionHtml(html)).not.toContain("Excluded");
    });

    it("extracts story when the Japanese heading and body are in the same block", () => {
      const html = `
        <p>\u30b9\u30c8\u30fc\u30ea\u30fc${fanzaStory}</p>
        <p>\u30b9\u30af\u30ea\u30fc\u30f3\u30b7\u30e7\u30c3\u30c8</p>
        <p>Excluded screenshot text.</p>
      `;

      const description = extractStorySectionHtml(html);

      expect(description).toBe(fanzaStory);
      expect(description).not.toContain("Excluded screenshot text");
    });

    it("cuts story extraction before the appearing character section", () => {
      const html = `
        <h2>\u3042\u3089\u3059\u3058</h2>
        <p>${fanzaStory}</p>
        <h2>\u767b\u5834\u30ad\u30e3\u30e9\u30af\u30bf\u30fc</h2>
        <p>\u30d2\u30ed\u30a4\u30f3\u7d39\u4ecb\u306f\u8aac\u660e\u6587\u306b\u542b\u3081\u306a\u3044\u3002</p>
      `;

      const description = extractStorySectionHtml(html);

      expect(description).toContain(fanzaStory);
      expect(description).not.toContain(
        "\u30d2\u30ed\u30a4\u30f3\u7d39\u4ecb"
      );
    });

    it("cuts story extraction before trailing notices and screenshots", () => {
      const html = `
        <h2>\u3042\u3089\u3059\u3058</h2>
        <p>${fanzaStory}</p>
        <p>\u203b\u672c\u4f5c\u306f\u300c\u65e5\u672c\u8a9e\u3001\u82f1\u8a9e\u3001\u4e2d\u56fd\u8a9e(\u7c21\u4f53\u5b57)\u300d\u306b\u5bfe\u5fdc\u3057\u3066\u304a\u308a\u307e\u3059\u3002</p>
        <h2>\u30b9\u30af\u30ea\u30fc\u30f3\u30b7\u30e7\u30c3\u30c8</h2>
        <p>Excluded screenshot text.</p>
      `;

      const description = extractStorySectionHtml(html);

      expect(description).toContain(fanzaStory);
      expect(description).not.toContain("\u5bfe\u5fdc\u3057\u3066\u304a\u308a\u307e\u3059");
      expect(description).not.toContain("Excluded screenshot text");
    });

    it("cuts story extraction before concept and staff-credit-like trailing blocks", () => {
      const html = `
        <h2>\u3042\u3089\u3059\u3058</h2>
        <p>${fanzaStory}</p>
        <h2>\u30b3\u30f3\u30bb\u30d7\u30c8</h2>
        <p>Excluded concept text.</p>
      `;
      const fanzaHtml = `
        <p>${fanzaStory}</p>
        <p>\u30fb\u51ea\u9593\u3086\u3081\u307f\u4f5c\u4e2d\u30a4\u30e9\u30b9\u30c8\u62c5\u5f53\u3000\u6c38\u5c71\u3086\u3046\u306e\u3093</p>
      `;

      expect(extractStorySectionHtml(html)).toBe(fanzaStory);
      expect(extractStorySectionHtml(fanzaHtml, { allowFallbackFromStart: true })).toBe(
        fanzaStory,
      );
    });

    it("extracts FANZA story from headings or narrative lead text", () => {
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <h2>\u30b9\u30c8\u30fc\u30ea\u30fc</h2>
              <p>${fanzaStory}</p>
              <h2>\u30b2\u30fc\u30e0\u5185\u5bb9</h2>
              <p>Excluded system text.</p>
            </div>
          </section>
        `,
        "text/html",
      );
      const noHeadingDoc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area"><p class="text-overflow">${fanzaStory}</p></div>
            </div>
          </section>
        `,
        "text/html",
      );

      expect(getFanzaDescriptionFromDocument(doc)).toBe(fanzaStory);
      expect(getFanzaDescriptionFromDocument(noHeadingDoc)).toBe(fanzaStory);
    });

    it("skips FANZA promotion lead text before a story heading", () => {
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  \u2605\u2606\u2605\u8c6a\u83ef\u7248\u7279\u5178\u5185\u5bb9\u2605\u2606\u2605<br>
                  \u58c1\u7d19\u304c\u3064\u3044\u3066\u304f\u308b\uff01<br>
                  \u25cb\u3042\u3089\u3059\u3058<br>
                  ${fanzaStory}<br>
                  \u30ad\u30e3\u30e9\u30af\u30bf\u30fc<br>
                  Excluded character text.
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );

      expect(getFanzaDescriptionFromDocument(doc)).toBe(fanzaStory);
    });

    it("skips FANZA promotion lead text before narrative text without a heading", () => {
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  \u2606\u2605\u2606\u30c0\u30a6\u30f3\u30ed\u30fc\u30c9\u7248\u5c02\u7528\u58c1\u7d19\u4ed8\u304d\u2606\u2605\u2606<br>
                  \u672c\u7de8\u300c\u30a2\u30e1\u30a4\u30b8\u30f3\u30b0\u30fb\u30b0\u30ec\u30a4\u30b9\u300d\u306b\u3001<br>
                  \u63cf\u304d\u4e0b\u308d\u3057\u58c1\u7d19\u304c\u4ed8\u3044\u305f\u8c6a\u83ef\u4ed5\u69d8\uff01\uff01<br>
                  \u2501\u2501\u30fb\u2025\u2026\u2501\u2501\u30fb\u2025\u2026\u2501\u2501<br>
                  \u300c\u611b\u3057\u306e\u541b\u3078 \u660e\u3051\u306a\u3044\u8056\u591c\u306e\u795d\u798f\u3092\u2500\u2500\u300d<br>
                  ${fanzaStory}
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );
      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(fanzaStory);
      expect(description).toContain("\u611b\u3057\u306e\u541b\u3078");
      expect(description).not.toContain("\u58c1\u7d19");
      expect(description).not.toContain("\u2501\u2501");
    });

    it("skips FANZA limited edition and soundtrack notices before narrative text", () => {
      const storyStart =
        "\u5927\u56fd\u306e\u7de9\u885d\u5730\u5e2f\u306b\u8a2d\u5b9a\u3055\u308c\u305f\u4e2d\u7acb\u7279\u533a\u304c\u7269\u8a9e\u306e\u821e\u53f0\u3067\u3059\u3002";
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  \u2605\u2606\u2605\u8c6a\u83ef\u9650\u5b9a\u7248\u306e\u5185\u5bb9\u306f\u3053\u3061\u3089\u2605\u2606\u2605<br>
                  \u25c6\u8a2d\u5b9a\u8cc7\u6599\u96c6\u30c7\u30fc\u30bf<br>
                  \u30ad\u30e3\u30e9\u30af\u30bf\u30fc\u7d39\u4ecb\u3084\u30e9\u30d5\u30a4\u30e9\u30b9\u30c8\u3092\u63b2\u8f09\u3002<br>
                  \u25c6\u30aa\u30ea\u30b8\u30ca\u30eb\u30b5\u30a6\u30f3\u30c9\u30c8\u30e9\u30c3\u30afCD\u97f3\u6e90<br>
                  \u2501\u2501\u30fb\u2025\u2026\u2501\u2501\u30fb\u2025\u2026\u2501\u2501<br>
                  ${storyStart}<br>
                  ${fanzaStory}
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );
      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(storyStart);
      expect(description).toContain(fanzaStory);
      expect(description).not.toContain("\u8c6a\u83ef\u9650\u5b9a\u7248");
      expect(description).not.toContain("\u30b5\u30a6\u30f3\u30c9\u30c8\u30e9\u30c3\u30af");
    });

    it("skips FANZA soundtrack edition notices before narrative text", () => {
      const storyStart =
        "\u4eba\u985e\u306f\u540c\u3058\u5922\u3092\u898b\u308b\u3088\u3046\u306b\u306a\u3063\u305f\u3002";
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  \u300e\u767d\u663c\u5922\u306e\u9752\u5199\u771f\u300f\u306e\u30aa\u30ea\u30b8\u30ca\u30eb\u30b5\u30a6\u30f3\u30c9\u30c8\u30e9\u30c3\u30af\u4ed8\u304d\u7248\u3082\u767b\u5834\uff01\uff01<br>
                  \u203b\u30b5\u30a6\u30f3\u30c9\u30c8\u30e9\u30c3\u30af\u306f\u30c7\u30fc\u30bf\u3067\u53ce\u9332\u3055\u308c\u307e\u3059\u3002<br>
                  \u2501\u2501\u30fb\u2025\u2026\u2501\u2501\u30fb\u2025\u2026\u2501\u2501<br>
                  ${storyStart}<br>
                  ${fanzaStory}
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );
      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(storyStart);
      expect(description).toContain(fanzaStory);
      expect(description).not.toContain("\u30b5\u30a6\u30f3\u30c9\u30c8\u30e9\u30c3\u30af");
    });

    it("skips FANZA bonus voice drama preludes before the main story", () => {
      const bonusStory =
        "\u30eb\u30d3\u30a4\u3068H\u306b\u540c\u68f2\u4e2d\u306e\u30dc\u30a4\u30b9\u30c9\u30e9\u30de\u3067\u3059\u3002";
      const mainStory =
        "\u30d5\u30ea\u30ae\u30a2\u738b\u7acb\u30b8\u30e5\u30a8\u30ea\u30fc\u30fb\u30a2\u30ab\u30c7\u30df\u30a2\u3067\u306e\u7269\u8a9e\u3067\u3059\u3002";
      const relatedNotice =
        "\u4e00\u7dd2\u306b\u3084\u308c\u3070\u3088\u308a\u697d\u3057\u3081\u308b\uff01";
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  \u2605\u2606\u2605FANZA\u9650\u5b9a\u30dc\u30a4\u30b9\u30b3\u30f3\u30c6\u30f3\u30c4\u3064\u304d\uff01\u2605\u2606\u2605<br>
                  \u25c6\u30dc\u30a4\u30b9\u30c9\u30e9\u30de<br>
                  \u3042\u3089\u3059\u3058\uff1a<br>
                  ${bonusStory}<br>
                  \u25c6FANZA GAMES\u9650\u5b9a\u30b7\u30b9\u30c6\u30e0\u30dc\u30a4\u30b9<br>
                  \u30b7\u30b9\u30c6\u30e0\u30dc\u30a4\u30b9\u96c6<br>
                  \u2501\u2501\u30fb\u2025\u2026\u2501\u2501\u30fb\u2025\u2026\u2501\u2501<br>
                  ${mainStory}<br>
                  ${fanzaStory}<br>
                  \u2501\u2501\u30fb\u2025\u2026\u2501\u2501\u30fb\u2025\u2026\u2501\u2501<br>
                  ${relatedNotice}
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );
      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(mainStory);
      expect(description).toContain(fanzaStory);
      expect(description).not.toContain(bonusStory);
      expect(description).not.toContain(relatedNotice);
      expect(description).not.toContain("\u30b7\u30b9\u30c6\u30e0\u30dc\u30a4\u30b9");
    });

    it("skips FANZA update patch notices before narrative text", () => {
      const storyStart =
        "\u4eca\u5e74\u304b\u3089\u5f93\u59c9\u59b9\u306e\u5bb6\u306b\u79fb\u308a\u4f4f\u3080\u3053\u3068\u306b\u306a\u3063\u305f\u4e3b\u4eba\u516c\u306e\u7269\u8a9e\u3067\u3059\u3002";
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  \u30102022/10/14 \u30a2\u30da\u30f3\u30c9\uff0b\u4fee\u6b63\u30d1\u30c3\u30c1\u914d\u5e03\u3011<br>
                  \u30fb\u4e0d\u5177\u5408\u3092\u4fee\u6b63\u3057\u307e\u3057\u305f\u3002<br>
                  \u203b\u30d0\u30fc\u30b8\u30e7\u30f3\u60c5\u5831\u304c1.01\u306b\u306a\u308a\u307e\u3059\u3002<br>
                  \u2501\u2501\u30fb\u2025\u2026\u2501\u2501\u30fb\u2025\u2026\u2501\u2501<br>
                  ${storyStart}<br>
                  ${fanzaStory}
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );
      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(storyStart);
      expect(description).toContain(fanzaStory);
      expect(description).not.toContain("\u4fee\u6b63\u30d1\u30c3\u30c1");
      expect(description).not.toContain("\u4e0d\u5177\u5408");
    });

    it("cuts FANZA related product notices even without a separator", () => {
      const relatedNotice =
        "\u65b0\u898f\u653b\u7565\u30d2\u30ed\u30a4\u30f3\u304c\u767b\u5834\u3001\u7d9a\u7de8\u4f5c\u306f\u3053\u3061\u3089";
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  ${fanzaStory}<br>
                  ${relatedNotice}
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );
      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(fanzaStory);
      expect(description).not.toContain(relatedNotice);
    });

    it("keeps all FANZA text-overflow story blocks instead of only the first one", () => {
      const continuation =
        "\u7d9a\u304f\u672c\u6587\u3067\u3059\u3002\u4e8c\u3064\u76ee\u306e\u6bb5\u843d\u3082\u8aac\u660e\u3068\u3057\u3066\u8868\u793a\u3057\u307e\u3059\u3002";
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">${fanzaStory}</p>
                <p class="text-overflow">${continuation}</p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );

      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(fanzaStory);
      expect(description).toContain(continuation);
    });

    it("stops FANZA narrative lead extraction at long decorative separators", () => {
      const continuation =
        "\u533a\u5207\u308a\u7dda\u306e\u5f8c\u306b\u7d9a\u304f\u7269\u8a9e\u306e\u672c\u6587\u3067\u3059\u3002";
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  ${fanzaStory}<br>
                  \u2501\u2501\u2501\u2501\u2501\u2501\u2501\u2501\u2501\u2501<br>
                  ${continuation}
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );

      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(fanzaStory);
      expect(description).not.toContain(continuation);
      expect(description).not.toContain("\u2501\u2501");
    });

    it("keeps FANZA text after short decorative separators", () => {
      const continuation =
        "\u77ed\u3044\u533a\u5207\u308a\u8a18\u53f7\u306e\u5f8c\u306b\u7d9a\u304f\u672c\u6587\u3067\u3059\u3002";
      const doc = new DOMParser().parseFromString(
        `
          <section class="universalSection">
            <div class="area-detail-read">
              <div class="read-text-area">
                <p class="text-overflow">
                  ${fanzaStory}<br>
                  ----<br>
                  ${continuation}
                </p>
              </div>
            </div>
          </section>
        `,
        "text/html",
      );
      const description = getFanzaDescriptionFromDocument(doc);

      expect(description).toContain(fanzaStory);
      expect(description).toContain(continuation);
      expect(description).not.toContain("----");
    });

    it("extracts and sanitizes DLsite story from itemprop description only", () => {
      const doc = new DOMParser().parseFromString(
        `
          <div itemprop="description">
            <script>alert("x")</script>
            <style>.x { color: red; }</style>
            <h2>\u3042\u3089\u3059\u3058</h2>
            <p onclick="evil()" style="color: red; background-color: black; font-weight: bold;">${dlsiteStory}</p>
            <a href="javascript:alert(1)">Bad link</a>
            <a href="//example.test/work">Good link</a>
            <font color="red">Font text</font>
            <img src="//img.example.test/sample.jpg">
            <h2>\u30b7\u30b9\u30c6\u30e0</h2>
            <p>Excluded system text.</p>
          </div>
        `,
        "text/html",
      );
      const html = getDlsiteDescriptionHtmlFromDocument(
        doc,
        "https://www.dlsite.com/maniax/work/=/product_id/RJ123456.html?locale=ja",
      );

      expect(html).toContain(dlsiteStory);
      expect(html).not.toContain("<script");
      expect(html).not.toContain("<style");
      expect(html).not.toContain("onclick");
      expect(html).not.toContain("javascript:");
      expect(html).not.toContain("color:");
      expect(html).not.toContain("background-color");
      expect(html).not.toContain("color=\"red\"");
      expect(html).not.toContain("<img");
      expect(html).toContain('href="https://example.test/work"');
      expect(html).not.toContain("Excluded system text");
      expect(getDlsiteDescriptionFromDocument(doc)).toContain(dlsiteStory);
    });

    it("does not save DLsite body text when no story heading exists", () => {
      const doc = new DOMParser().parseFromString(
        `<div itemprop="description"><p>${dlsiteStory}</p></div>`,
        "text/html",
      );

      expect(getDlsiteDescriptionFromDocument(doc)).toBeUndefined();
    });

    it("falls back to the DLsite announce page for a confirmed product ID", async () => {
      vi.mocked(tauriHttpPluginFetch).mockImplementation(tauriHttpFetch as any);

      const result = await fetchFromDlsite({
        title:
          "\u30a2\u30a4\u30f3\u30b7\u30e5\u30bf\u30a4\u30f3\u3088\u308a\u611b\u3092\u8fbc\u3081\u3066",
        externalIds: {
          dlsiteId: "VJ013711",
          dlsiteDomain: "pro",
        },
      });

      expect(result?.sourceUrl).toBe(createDlsiteAnnounceUrl("VJ013711", "pro"));
      expect(result?.descriptionHtml).toContain(
        "\u4eba\u306e\u6c17\u6301\u3061\u304c\u5206\u304b\u3089\u306a\u3044"
      );
      expect(result?.descriptionHtml).not.toContain(
        "\u30ad\u30e3\u30e9\u30af\u30bf\u30fc\u7d39\u4ecb"
      );
    });

    it("accepts DLsite Concept as a story heading and stops at Character", () => {
      const doc = new DOMParser().parseFromString(
        `
          <div itemprop="description">
            <h2>Concept</h2>
            <p>${dlsiteStory}</p>
            <h2>Character</h2>
            <p>Excluded character text.</p>
          </div>
        `,
        "text/html",
      );

      expect(getDlsiteDescriptionFromDocument(doc)).toBe(dlsiteStory);
    });

    it("uses Steam appdetails story/about/short order and cuts mature/system sections", () => {
      expect(
        getSteamDescriptionFromAppDetails(
          {
            "1": {
              success: true,
              data: {
                detailed_description: `
                  <h2>\u30b9\u30c8\u30fc\u30ea\u30fc</h2>
                  <p>${steamStory}</p>
                  <h2>Mature Content Description</h2>
                  <p>Excluded mature text.</p>
                  <h2>System Requirements</h2>
                  <p>Excluded system text.</p>
                `,
                about_the_game: `<p>${steamAbout}</p>`,
                short_description: steamShort,
              },
            },
          },
          1,
        ),
      ).toBe(steamStory);

      expect(
        getSteamDescriptionFromAppDetails(
          {
            "2": {
              success: true,
              data: {
                detailed_description: "<h2>Features</h2><p>Feature list.</p>",
                about_the_game: `<p>${steamAbout}</p><h2>System Requirements</h2><p>Excluded.</p>`,
                short_description: steamShort,
              },
            },
          },
          2,
        ),
      ).toBe(steamAbout);

      expect(
        getSteamDescriptionFromAppDetails(
          {
            "3": {
              success: true,
              data: {
                detailed_description: `
                  <p>\u25c6\u30b9\u30c8\u30fc\u30ea\u30fc\u300c${steamStory}\u300d</p>
                  <p>\u25a0\u30b2\u30fc\u30e0\u7d39\u4ecb</p>
                  <p>Excluded game introduction.</p>
                `,
              },
            },
          },
          3,
        ),
      ).toBe(`\u300c${steamStory}\u300d`);

      expect(
        getSteamDescriptionFromAppDetails(
          {
            "4": {
              success: true,
              data: {
                detailed_description: `
                  <p>\uff1cSTORY\uff1e</p>
                  <p>${steamStory}</p>
                  <p>System Requirements</p>
                  <p>Excluded system text.</p>
                `,
              },
            },
          },
          4,
        ),
      ).toBe(steamStory);

      expect(
        getSteamDescriptionFromAppDetails(
          {
            "5": {
              success: true,
              data: {
                detailed_description: `
                  <p>\u672c\u4f5c\u306f\u3042\u304f\u307e\u3067\u30d3\u30b8\u30e5\u30a2\u30eb\u30ce\u30d9\u30eb\u3067\u3059\u3002<br>
                  \u30b9\u30c8\u30fc\u30ea\u30fc\u3088\u308a\u3082\u8b0e\u89e3\u304d\u30d1\u30ba\u30eb\u3092\u91cd\u8996\u3059\u308b\u65b9\u306b\u306f\u9069\u3057\u3066\u3044\u306a\u3044\u304b\u3082\u3057\u308c\u307e\u305b\u3093\u3002</p>
                  <h2>\u30b9\u30c8\u30fc\u30ea\u30fc</h2>
                  <p>${steamStory}</p>
                  <h2>\u30b9\u30af\u30ea\u30fc\u30f3\u30b7\u30e7\u30c3\u30c8</h2>
                  <p>Excluded screenshot text.</p>
                `,
              },
            },
          },
          5,
        ),
      ).toBe(steamStory);

      expect(
        getSteamDescriptionFromAppDetails(
          {
            "6": {
              success: true,
              data: {
                about_the_game: `
                  <p>\u3010STORY\u3011</p>
                  <p>${steamStory}</p>
                  <p>\u3010STAFF\u3011</p>
                  <p>Excluded staff text.</p>
                `,
              },
            },
          },
          6,
        ),
      ).toBe(steamStory);
    });

    it("does not accept non-Japanese Steam descriptions as Japanese text", () => {
      expect(
        getSteamDescriptionFromAppDetails(
          {
            "1": {
              success: true,
              data: {
                detailed_description:
                  "\u83ba\u795e\u4e50\u7684\u77e5\u540d\u4f5c\u54c1\uff0c\u5728Steam\u4e2d\u4e0a\u67b6\uff01",
                short_description: "English story text.",
              },
            },
          },
          1,
        ),
      ).toBeUndefined();
    });

    it("uses gamelist sales IDs for direct fetch even when ErogeScape HTML has no store links", async () => {
      vi.mocked(tauriHttpPluginFetch).mockImplementation(tauriHttpFetch as any);
      const seed = getWorkById(20988);
      expect(seed).not.toBeNull();
      const minimalErogamescapeHtml = `
        <html>
          <body>
            <h1 id="game_title"><a href="${seed?.officialHomePage}">${seed?.name}</a></h1>
            <div id="soft-title">
              <a href="brand.php?brand=${seed?.brandId}">${seed?.brandName}</a>
              <a>${seed?.sellday}</a>
            </div>
            <div id="main_image"><img src="${seed?.imgUrl}" /></div>
            <table>
              <tr id="median"><td>0</td></tr>
              <tr id="count"><td>0</td></tr>
              <tr id="average"><td>0</td></tr>
              <tr id="play_time"><td>0</td></tr>
              <tr id="genga"><td></td></tr>
              <tr id="shinario"><td></td></tr>
              <tr id="seiyu"><td></td></tr>
            </table>
            <table id="music_summary_main"></table>
          </body>
        </html>
      `;

      vi.spyOn(window, "fetch").mockImplementation(async (url) => {
        const urlString = String(url);
        if (urlString.includes("20988.html")) {
          return { ok: true, text: async () => minimalErogamescapeHtml } as any;
        }
        return { ok: false, text: async () => "" } as any;
      });

      const work = await getWorkByScrape(20988);
      const requestedUrls = vi
        .mocked(tauriHttpPluginFetch)
        .mock.calls.map(([url]) => String(url));
      const fanzaRequest = vi
        .mocked(tauriHttpPluginFetch)
        .mock.calls.find(([url]) =>
          String(url).includes("https://dlsoft.dmm.co.jp/detail/nightingale_0001/"),
        )?.[1];

      expect(requestedUrls).toContain(
        "https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/sql_for_erogamer_form.php",
      );
      expect(requestedUrls).toContain(
        "https://dlsoft.dmm.co.jp/detail/nightingale_0001/",
      );
      expect(fanzaRequest?.headers).toMatchObject({
        Cookie: expect.stringContaining("age_check_done=1"),
        Referer: "https://dlsoft.dmm.co.jp/",
      });
      expect(work.description).toContain("FANZA\u30b9\u30c8\u30fc\u30ea\u30fc\u672c\u6587");
      expect(work.description).not.toContain("\u30ad\u30e3\u30e9\u30af\u30bf\u30fc\u7d39\u4ecb");
    });

    it("uses the official site only to discover a Steam app ID when gamelist steam is empty", async () => {
      vi.mocked(tauriHttpPluginFetch).mockImplementation(tauriHttpFetch as any);
      vi.spyOn(window, "fetch").mockImplementation(async (url) => {
        if (String(url).includes("37755.html")) {
          return {
            ok: true,
            text: async () => `
              <html>
                <body>
                  <h1 id="game_title"><a href="https://virtualgirl.bushiroadgames.com/">VIRTUAL GIRL @ WORLD'S END</a></h1>
                  <div id="soft-title">
                    <a href="brand.php?brand=2844">\u30d6\u30b7\u30ed\u30fc\u30c9</a>
                    <a>2025-06-12</a>
                  </div>
                  <div id="main_image"><img src="" /></div>
                  <table>
                    <tr id="median"><td>0</td></tr>
                    <tr id="count"><td>0</td></tr>
                    <tr id="average"><td>0</td></tr>
                    <tr id="play_time"><td>0</td></tr>
                    <tr id="genga"><td></td></tr>
                    <tr id="shinario"><td></td></tr>
                    <tr id="seiyu"><td></td></tr>
                  </table>
                  <table id="music_summary_main"></table>
                </body>
              </html>
            `,
          } as any;
        }
        return { ok: false, text: async () => "" } as any;
      });

      const work = await getWorkByScrape(37755);
      const requestedUrls = vi
        .mocked(tauriHttpPluginFetch)
        .mock.calls.map(([url]) => String(url));

      expect(requestedUrls).toContain("https://virtualgirl.bushiroadgames.com/");
      expect(requestedUrls).toContain(
        "https://store.steampowered.com/api/appdetails?appids=3290440&l=japanese&cc=JP"
      );
      expect(work.description).toContain("Steam\u30b9\u30c8\u30fc\u30ea\u30fc\u672c\u6587");
      expect(work.brandName).toBe("\u30d6\u30b7\u30ed\u30fc\u30c9");
    });
  });
});
