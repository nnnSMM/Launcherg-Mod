import {
  getCollectionElementSeed,
  getDetailSeedById,
  getGameCacheById,
  getWorkById,
} from "@/mock/demoCatalog";

type FetchOptions = {
  method?: string;
  headers?: Record<string, string>;
  body?: unknown;
};

const textResponse = (text: string) => ({
  ok: true,
  text: async () => text,
  json: async () => JSON.parse(text || "{}"),
  arrayBuffer: async () => new TextEncoder().encode(text).buffer,
});

const escapeHtml = (value: string | number | boolean | null | undefined) =>
  String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

const readSql = (options?: FetchOptions) => {
  const body = options?.body;
  if (body instanceof FormData) {
    return String(body.get("sql") ?? "");
  }
  return "";
};

const extractInIds = (sql: string) => {
  const inMatch = sql.match(/\bin\s*\(([^)]+)\)/i);
  if (!inMatch) {
    return [];
  }
  return [...inMatch[1].matchAll(/\d+/g)].map(([value]) => Number(value));
};

const extractSqlIds = (sql: string) => {
  const inIds = extractInIds(sql);
  if (inIds.length) return inIds;
  const idMatch = sql.match(/\b(?:gamelist\.)?id\s*=\s*'?(\d+)'?/i);
  return idMatch ? [Number(idMatch[1])] : [];
};

const queryTable = (
  rows: Array<Array<string | number | boolean | null | undefined>>,
) => `
  <table id="query_result_main">
    <tr><th></th></tr>
    ${rows
      .map(
        (row) =>
          `<tr>${row.map((value) => `<td>${escapeHtml(value)}</td>`).join("")}</tr>`,
      )
      .join("")}
  </table>
`;

const buildSqlResponse = (sql: string) => {
  const ids = extractSqlIds(sql);

  if (
    /\bdmm\b/i.test(sql) &&
    /\bdlsite_id\b/i.test(sql) &&
    /\bdlsite_domain\b/i.test(sql) &&
    /\bsteam\b/i.test(sql)
  ) {
    const includesShoukai = /\bshoukai\b/i.test(sql);
    return queryTable(
      ids.slice(0, 1).map((id) => {
        const dmm = findKeyByValue(fanzaProductToGameId, id) ?? "";
        const dlsiteId = findKeyByValue(dlsiteProductToGameId, id) ?? "";
        const steam = id === 37755 ? "" : findKeyByValue(steamAppToGameId, id) ?? "";
        const dlsiteDomain = dlsiteId
          ? dlsiteId.startsWith("VJ")
            ? "pro"
            : "maniax"
          : "";
        const shoukai = officialSiteByGameId.get(id) ?? "";
        return includesShoukai
          ? [dmm, dlsiteId, dlsiteDomain, steam, shoukai]
          : [dmm, dlsiteId, dlsiteDomain, steam];
      }),
    );
  }

  if (/brandlist/i.test(sql) || /furigana/i.test(sql)) {
    return queryTable(
      ids.map((id) => {
        const detail = getDetailSeedById(id);
        return [
          id,
          detail.gamenameRuby,
          detail.sellday,
          detail.isNukige ? "t" : "f",
          detail.brandname,
          detail.brandnameRuby,
        ];
      }),
    );
  }

  return queryTable(
    ids
      .map((id) => getGameCacheById(id))
      .filter(Boolean)
      .map((cache) => [cache?.id, cache?.gamename, cache?.thumbnailUrl]),
  );
};

const buildGamePage = (id: number) => {
  const work = getWorkById(id);
  if (!work) {
    return "";
  }
  const fanzaProductId = findKeyByValue(fanzaProductToGameId, id);
  const dlsiteProductId = findKeyByValue(dlsiteProductToGameId, id);
  const steamAppId = id === 37755 ? undefined : findKeyByValue(steamAppToGameId, id);

  return `
    <html>
      <body>
        <h1 id="game_title"><a href="${escapeHtml(work.officialHomePage)}">${escapeHtml(work.name)}</a></h1>
        <div id="soft-title">
          <a href="brand.php?brand=${work.brandId}">${escapeHtml(work.brandName)}</a>
          <a>${escapeHtml(work.sellday)}</a>
        </div>
        <div id="main_image"><img src="${escapeHtml(work.imgUrl)}" /></div>
        <table>
          <tr id="median"><td>${work.statistics.median}</td></tr>
          <tr id="count"><td>${work.statistics.count}</td></tr>
          <tr id="average"><td>${work.statistics.average}</td></tr>
          <tr id="play_time"><td>${escapeHtml(work.statistics.playTime)}</td></tr>
          <tr id="genga"><td></td></tr>
          <tr id="shinario"><td></td></tr>
          <tr id="seiyu"><td></td></tr>
        </table>
        <table id="music_summary_main"></table>
        <div id="bottom_inter_links">
          ${fanzaProductId ? `<a href="https://al.fanza.co.jp/?lurl=${encodeURIComponent(`https://dlsoft.dmm.co.jp/detail/${fanzaProductId}/`)}">DMM</a>` : ""}
          ${dlsiteProductId ? `<a href="https://www.dlsite.com/soft/dlaf/=/link/work/aid/erogamescape/id/${dlsiteProductId}.html">DLsite.com</a>` : ""}
          ${steamAppId ? `<a href="https://store.steampowered.com/app/${steamAppId}/">STEAM</a>` : ""}
        </div>
      </body>
    </html>
  `;
};

const steamAppToGameId = new Map<number, number>([
  [3101040, 38631],
  [3782920, 38696],
  [3290440, 37755],
]);

const fanzaProductToGameId = new Map<string, number>([
  ["nightingale_0001", 20988],
  ["vsat_0229", 25861],
  ["favorite_0011", 26245],
  ["cabbage_0003", 27059],
  ["hobe_0494", 28941],
  ["nightingale_0005", 30122],
  ["akbs_0127", 31106],
  ["vsat_0288", 31597],
  ["yuzu_0012", 38794],
  ["spal_0201", 39837],
]);

const dlsiteProductToGameId = new Map<string, number>([
  ["VJ015604", 31597],
]);

const findKeyByValue = <T, U>(map: Map<T, U>, value: U) =>
  [...map.entries()].find(([, candidate]) => candidate === value)?.[0];

const officialSiteByGameId = new Map<number, string>([
  [37755, "https://virtualgirl.bushiroadgames.com/"],
]);

const buildVirtualGirlOfficialPage = () => `
  <html>
    <body>
      <h1>VIRTUAL GIRL @ WORLD'S END</h1>
      <a href="https://store.steampowered.com/app/3290440/">Steam Store</a>
    </body>
  </html>
`;

const descriptionToStoryHtml = (description: string) =>
  description
    .split("\n")
    .map((line) => (line.trim() ? `<p>${escapeHtml(line)}</p>` : "<br>"))
    .join("");

const buildFanzaProductPage = (productId: string) => {
  const gameId = fanzaProductToGameId.get(productId);
  const work = gameId ? getWorkById(gameId) : null;
  if (!work) return "";

  const storyBody = work.description
    ? descriptionToStoryHtml(work.description)
    : `<p>${escapeHtml(work.name)}\u306eFANZA\u30b9\u30c8\u30fc\u30ea\u30fc\u672c\u6587\u3067\u3059\u3002</p>`;

  return `
    <html>
      <body>
        <h1 class="productTitle__item">${escapeHtml(work.name)}</h1>
        <div class="contentsDetailTop__table">
          <div class="contentsDetailTop__tableDataLeft">\u30d6\u30e9\u30f3\u30c9</div>
          <div class="contentsDetailTop__tableDataRight">${escapeHtml(work.brandName)}</div>
        </div>
        <section class="universalSection">
          <div class="area-detail-read">
            <h2>\u30b9\u30c8\u30fc\u30ea\u30fc</h2>
            ${storyBody}
            <h2>\u30ad\u30e3\u30e9\u30af\u30bf\u30fc</h2>
            <p>\u3053\u3053\u306f\u4fdd\u5b58\u3057\u306a\u3044\u30ad\u30e3\u30e9\u30af\u30bf\u30fc\u7d39\u4ecb\u3067\u3059\u3002</p>
          </div>
        </section>
      </body>
    </html>
  `;
};

const buildDlsiteProductPage = (productId: string) => {
  const gameId = dlsiteProductToGameId.get(productId);
  const work = gameId ? getWorkById(gameId) : null;
  if (!work) return "";

  return `
    <html>
      <body>
        <h1 id="work_name">${escapeHtml(work.name)}</h1>
        <div class="maker_name"><a>${escapeHtml(work.brandName)}</a></div>
        <div itemprop="description">
          <h2>\u3042\u3089\u3059\u3058</h2>
          <p>${escapeHtml(work.name)}\u306eDLsite\u30b9\u30c8\u30fc\u30ea\u30fc\u672c\u6587\u3067\u3059\u3002\u4f5c\u54c1\u306e\u5c0e\u5165\u3092\u65e5\u672c\u8a9e\u3067\u63b2\u8f09\u3057\u3066\u3044\u307e\u3059\u3002</p>
          <h2>\u30b7\u30b9\u30c6\u30e0</h2>
          <p>\u3053\u3053\u306f\u4fdd\u5b58\u3057\u306a\u3044\u30b7\u30b9\u30c6\u30e0\u8aac\u660e\u3067\u3059\u3002</p>
        </div>
        <div id="dlsite_sample_cg_main">
          <img src="https://img.dlsite.jp/resize/images2/work/professional/VJ016000/${productId}_img_smpa1_200x200.jpg">
          <img src="https://img.dlsite.jp/resize/images2/work/professional/VJ016000/${productId}_img_smpa2_200x200.jpg">
          <img src="https://img.dlsite.jp/resize/images2/work/professional/VJ016000/${productId}_img_smpa3_200x200.jpg">
          <img src="https://img.dlsite.jp/resize/images2/work/professional/VJ016000/${productId}_img_smpa4_200x200.jpg">
        </div>
      </body>
    </html>
  `;
};

const buildDlsiteAnnouncePage = (productId: string) => {
  if (productId !== "VJ013711") return "";
  return `
    <html>
      <body>
        <h1 id="work_name">\u30a2\u30a4\u30f3\u30b7\u30e5\u30bf\u30a4\u30f3\u3088\u308a\u611b\u3092\u8fbc\u3081\u3066</h1>
        <div class="maker_name"><a>GLOVETY</a></div>
        <div itemprop="description">
          <h2>\u30b9\u30c8\u30fc\u30ea\u30fc</h2>
          <p>\u4eba\u306e\u6c17\u6301\u3061\u304c\u5206\u304b\u3089\u306a\u3044\u5974\u3060\u3068\u8a00\u308f\u308c\u308b\u3002\u3060\u3051\u3069\u4eba\u306e\u6c17\u6301\u3061\u306a\u3093\u3066\u3001\u8ab0\u306b\u5206\u304b\u308b\u3068\u8a00\u3046\u306e\u3060\u308d\u3046\uff1f</p>
          <h2>\u30ad\u30e3\u30e9\u30af\u30bf\u30fc</h2>
          <p>\u3053\u3053\u306f\u4fdd\u5b58\u3057\u306a\u3044\u30ad\u30e3\u30e9\u30af\u30bf\u30fc\u7d39\u4ecb\u3067\u3059\u3002</p>
        </div>
      </body>
    </html>
  `;
};

const buildSteamAppDetails = (appId: number) => {
  const gameId = steamAppToGameId.get(appId);
  const work = gameId ? getWorkById(gameId) : null;
  if (!work) {
    return JSON.stringify({ [appId]: { success: false } });
  }

  return JSON.stringify({
    [appId]: {
      success: true,
      data: {
        name: work.name,
        detailed_description: `<h2>\u30b9\u30c8\u30fc\u30ea\u30fc</h2><p>${escapeHtml(work.name)}\u306eSteam\u30b9\u30c8\u30fc\u30ea\u30fc\u672c\u6587\u3067\u3059\u3002</p><h2>Mature Content Description</h2><p>Excluded mature content note.</p><h2>System Requirements</h2><p>Excluded system text.</p>`,
        about_the_game: `<p>${escapeHtml(work.name)}\u306eSteam About This Game\u3067\u3059\u3002</p>`,
        short_description: `${work.name}\u306eSteam\u77ed\u3044\u8aac\u660e\u3067\u3059\u3002`,
        developers: [work.brandName],
        publishers: [work.brandName],
        header_image: work.imgUrl,
        capsule_image: work.imgUrl,
        capsule_imagev5: work.imgUrl,
      },
    },
  });
};

const buildSteamProductPage = (appId: number) => {
  const gameId = steamAppToGameId.get(appId);
  if (!gameId) {
    return "";
  }

  const work = getWorkById(gameId);
  const collectionElement = getCollectionElementSeed(gameId);
  if (!work || !collectionElement) {
    return "";
  }

  const localPreviewUrl = new URL(
    `./${collectionElement.thumbnail}`,
    window.location.href,
  ).href;
  const coverUrl = work.imgUrl || localPreviewUrl;

  return `
    <html>
      <head>
        <meta property="og:image" content="${escapeHtml(coverUrl)}" />
        <meta name="twitter:image" content="${escapeHtml(localPreviewUrl)}" />
      </head>
      <body>
        <div class="apphub_AppName">${escapeHtml(work.name)}</div>
        <img src="${escapeHtml(coverUrl)}" alt="cover" />
        <img src="${escapeHtml(localPreviewUrl)}" alt="preview" />
      </body>
    </html>
  `;
};

export const fetch = async (url: string, options?: FetchOptions) => {
  console.log("[Mock Tauri HTTP] fetch:", url, options);

  if (/^https:\/\/virtualgirl\.bushiroadgames\.com\/?/i.test(url)) {
    return textResponse(buildVirtualGirlOfficialPage());
  }

  const steamAppDetailsMatch = url.match(
    /^https:\/\/store\.steampowered\.com\/api\/appdetails\?/i,
  );
  if (steamAppDetailsMatch) {
    try {
      const appId = Number(new URL(url).searchParams.get("appids") ?? "0");
      return textResponse(buildSteamAppDetails(appId));
    } catch {
      return textResponse("{}");
    }
  }

  const steamAppMatch = url.match(
    /^https:\/\/store\.steampowered\.com\/app\/(\d+)\/?[^?#]*/i,
  );
  if (steamAppMatch) {
    const appId = Number(steamAppMatch[1]);
    try {
      const res = await window.fetch(`./demo-data/steam-${appId}.html`);
      if (res.ok) {
        return textResponse(await res.text());
      }
    } catch (e) {
      console.warn(`Failed to fetch Steam demo html for ${appId}`, e);
    }
    const html = buildSteamProductPage(appId);
    if (html) {
      return textResponse(html);
    }
  }

  const fanzaDigitalProductMatch = url.match(
    /^https:\/\/dlsoft\.dmm\.(?:co\.jp|com)\/detail\/([^/?#]+)\/?/i,
  );
  if (fanzaDigitalProductMatch) {
    return textResponse(buildFanzaProductPage(fanzaDigitalProductMatch[1]));
  }

  const fanzaMonoProductMatch = url.match(
    /^https:\/\/www\.dmm\.(?:co\.jp|com)\/mono\/pcgame\/-\/detail\/=\/cid=([^/?#]+)\/?/i,
  );
  if (fanzaMonoProductMatch) {
    return textResponse(buildFanzaProductPage(fanzaMonoProductMatch[1]));
  }

  const fanzaDoujinProductMatch = url.match(
    /^https:\/\/www\.dmm\.(?:co\.jp|com)\/dc\/doujin\/-\/detail\/=\/cid=([^/?#]+)\/?/i,
  );
  if (fanzaDoujinProductMatch) {
    return textResponse(buildFanzaProductPage(fanzaDoujinProductMatch[1]));
  }

  const dlsiteProductMatch = url.match(
    /^https?:\/\/(?:www\.)?dlsite\.com\/[^/]+\/work\/=\/product_id\/([A-Z0-9]+)\.html/i,
  );
  if (dlsiteProductMatch) {
    const productId = dlsiteProductMatch[1];
    if (productId === "VJ013711") {
      return textResponse("");
    }
    try {
      const res = await window.fetch(`./demo-data/dlsite-${productId}.html`);
      if (res.ok) {
        return textResponse(await res.text());
      }
    } catch (e) {
      console.warn(`Failed to fetch DLsite demo html for ${productId}`, e);
    }
    const html = buildDlsiteProductPage(productId);
    if (html) {
      return textResponse(html);
    }
    return textResponse("");
  }

  const dlsiteAnnounceProductMatch = url.match(
    /^https?:\/\/(?:www\.)?dlsite\.com\/[^/]+\/announce\/=\/product_id\/([A-Z0-9]+)\.html/i,
  );
  if (dlsiteAnnounceProductMatch) {
    return textResponse(buildDlsiteAnnouncePage(dlsiteAnnounceProductMatch[1]));
  }

  if (
    url.includes("erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=")
  ) {
    const id = Number(url.split("game=")[1].split("&")[0]);
    try {
      const res = await window.fetch(`./demo-data/${id}.html`);
      if (res.ok) {
        return textResponse(await res.text());
      }
    } catch (e) {
      console.warn("Failed to fetch demo html, generating a minimal page", e);
    }
    return textResponse(buildGamePage(id));
  }

  if (url.includes("sql_for_erogamer_form.php")) {
    return textResponse(buildSqlResponse(readSql(options)));
  }

  if (url.includes("erogamescape")) {
    return textResponse(queryTable([]));
  }

  if (url.includes("game")) {
    return {
      ok: true,
      json: async () => ({ results: [] }),
      text: async () => "{}",
      arrayBuffer: async () => new TextEncoder().encode("{}").buffer,
    };
  }

  try {
    return await window.fetch(url, options as any);
  } catch {
    return {
      ok: true,
      text: async () => "",
      json: async () => ({}),
      arrayBuffer: async () => new ArrayBuffer(0),
    };
  }
};
