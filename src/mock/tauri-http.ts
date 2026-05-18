import { getDetailSeedById, getGameCacheById, getWorkById } from "@/mock/demoCatalog";

type FetchOptions = {
  method?: string;
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

const queryTable = (rows: Array<Array<string | number | boolean | null | undefined>>) => `
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
  const ids = extractInIds(sql);

  if (/brandlist/i.test(sql) || /furigana/i.test(sql)) {
    return queryTable(
      ids.map((id) => {
        const cache = getGameCacheById(id);
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
      .map((cache) => [
        cache?.id,
        cache?.gamename,
        cache?.thumbnailUrl,
      ]),
  );
};

const buildGamePage = (id: number) => {
  const work = getWorkById(id);
  if (!work) {
    return "";
  }

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
      </body>
    </html>
  `;
};

export const fetch = async (url: string, options?: FetchOptions) => {
  console.log("[Mock Tauri HTTP] fetch:", url, options);

  if (url.includes("erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=")) {
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

  if (url.includes("vndb")) {
    return {
      ok: true,
      json: async () => ({ results: [] }),
      text: async () => "{}",
      arrayBuffer: async () => new TextEncoder().encode("{}").buffer,
    };
  }

  return {
    ok: true,
    text: async () => "",
    json: async () => ({}),
    arrayBuffer: async () => new ArrayBuffer(0),
  };
};
