import {
  commandGetAppSetting,
  commandGetVndbScreenshotCache,
  commandUpsertVndbScreenshotCache,
} from "@/lib/command";
import type {
  CollectionElement,
  VndbScreenshot,
  VndbScreenshotCache,
} from "@/lib/types";
import { fetch } from "@tauri-apps/plugin-http";

const CACHE_TTL_MS = 30 * 24 * 60 * 60 * 1000;
const CACHE_SCHEMA_VERSION = 10;
const EROGAMESCAPE_REQUEST_PATH =
  "https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki";
const SENSITIVE_THRESHOLD = 1.5;
const PREFETCH_INTERVAL_MS = 2000;
const SHOW_SENSITIVE_SETTING_KEY = "show_sensitive_vndb_screenshots";

type VndbApiScreenshot = {
  id: string;
  url: string;
  thumbnail: string;
  dims?: [number, number] | null;
  thumbnail_dims?: [number, number] | null;
  sexual?: number | null;
  violence?: number | null;
  release?: {
    languages?: Array<{
      lang?: string | null;
      mtl?: boolean | null;
    }>;
  } | null;
};

type VndbApiTitle = {
  lang?: string | null;
  title?: string | null;
  latin?: string | null;
  official?: boolean | null;
  main?: boolean | null;
};

type VndbApiDeveloper = {
  name?: string | null;
  original?: string | null;
  aliases?: string[] | null;
};

type VndbApiResult = {
  id: string;
  title: string;
  alttitle?: string | null;
  titles?: VndbApiTitle[];
  aliases?: string[];
  developers?: VndbApiDeveloper[];
  released?: string | null;
  screenshots?: VndbApiScreenshot[];
};

type VndbApiResponse = {
  results?: VndbApiResult[];
};

type CacheLookup = {
  cache: VndbScreenshotCache | null;
  isFresh: boolean;
};

type VndbMatchContext = Pick<
  CollectionElement,
  "gamename" | "gamenameRuby" | "brandname" | "brandnameRuby" | "sellday"
>;

const inFlight = new Map<number, Promise<VndbScreenshotCache>>();
const queuedIds = new Set<number>();
const queue: CollectionElement[] = [];
let queueTimer: ReturnType<typeof setTimeout> | null = null;

const normalizeForMatch = (value: string | null | undefined) =>
  (value ?? "")
    .normalize("NFKC")
    .toLowerCase()
    .replace(
      /[\s"'`’‘“”.,:;!?！？。、・･~〜\-ー_()[\]{}【】「」『』《》〈〉<>]+/g,
      "",
    );

const uniqNonEmpty = (values: Array<string | null | undefined>) => [
  ...new Set(values.map((v) => v?.trim()).filter((v): v is string => !!v)),
];

const getCandidateTitles = (result: VndbApiResult) =>
  uniqNonEmpty([
    result.title,
    result.alttitle,
    ...(result.aliases ?? []),
    ...(result.titles ?? []).flatMap((title) => [title.title, title.latin]),
  ]);

const getDeveloperNames = (result: VndbApiResult) =>
  uniqNonEmpty(
    (result.developers ?? []).flatMap((developer) => [
      developer.name,
      developer.original,
      ...(developer.aliases ?? []),
    ]),
  );

const screenshotHasOnlyJapanese = (screenshot: VndbApiScreenshot) => {
  const languages = screenshot.release?.languages ?? [];
  return (
    languages.length > 0 &&
    languages.every((language) => language.lang === "ja" && !language.mtl)
  );
};

const screenshotHasJapanese = (screenshot: VndbApiScreenshot) => {
  const languages = screenshot.release?.languages ?? [];
  return languages.some((language) => language.lang === "ja" && !language.mtl);
};

const mapVndbScreenshot = (s: VndbApiScreenshot): VndbScreenshot => ({
      id: s.id,
      url: s.url,
      thumbnail: s.thumbnail,
      dims: s.dims ?? null,
      thumbnailDims: s.thumbnail_dims ?? null,
      sexual: s.sexual ?? 0,
      violence: s.violence ?? 0,
      languages: (s.release?.languages ?? [])
        .map((language) => language.lang)
        .filter((lang): lang is string => !!lang),
});

const parseJapaneseScreenshots = (result: VndbApiResult): VndbScreenshot[] => {
  const screenshots = (result.screenshots ?? []).filter(
    (s) => s.url && s.thumbnail,
  );
  const japaneseOnly = screenshots.filter(screenshotHasOnlyJapanese);
  if (japaneseOnly.length > 0) {
    return japaneseOnly.map(mapVndbScreenshot);
  }
  return screenshots.filter(screenshotHasJapanese).map(mapVndbScreenshot);
};

const titleMatchScore = (source: string, candidates: string[]) => {
  const normalizedSource = normalizeForMatch(source);
  if (!normalizedSource) return 0;

  return candidates.reduce((score, candidate) => {
    const normalizedCandidate = normalizeForMatch(candidate);
    if (!normalizedCandidate) return score;
    if (normalizedCandidate === normalizedSource) return Math.max(score, 100);
    if (
      normalizedCandidate.includes(normalizedSource) ||
      normalizedSource.includes(normalizedCandidate)
    ) {
      return Math.max(score, normalizedSource.length >= 4 ? 60 : 25);
    }
    return score;
  }, 0);
};

const brandMatchScore = (source: string, candidates: string[]) => {
  const normalizedSource = normalizeForMatch(source);
  if (!normalizedSource) return 0;

  return candidates.reduce((score, candidate) => {
    const normalizedCandidate = normalizeForMatch(candidate);
    if (!normalizedCandidate) return score;
    if (normalizedCandidate === normalizedSource) return Math.max(score, 35);
    if (
      normalizedCandidate.includes(normalizedSource) ||
      normalizedSource.includes(normalizedCandidate)
    ) {
      return Math.max(score, normalizedSource.length >= 3 ? 20 : 8);
    }
    return score;
  }, 0);
};

const releaseDateScore = (
  resultReleased: string | null | undefined,
  collectionReleased: string | null | undefined,
) => {
  if (!resultReleased || !collectionReleased) return 0;
  if (resultReleased === collectionReleased) return 100;
  if (resultReleased.slice(0, 7) === collectionReleased.slice(0, 7)) return 60;
  return resultReleased.slice(0, 4) === collectionReleased.slice(0, 4) ? 30 : 0;
};

const titleScoreForResult = (
  result: VndbApiResult,
  context: VndbMatchContext,
) => {
  const candidateTitles = getCandidateTitles(result);
  return Math.max(
    titleMatchScore(context.gamename, candidateTitles),
    titleMatchScore(context.gamenameRuby, candidateTitles),
  );
};

const tieBreakScoreForResult = (
  result: VndbApiResult,
  context: VndbMatchContext,
) => {
  const developerNames = getDeveloperNames(result);
  return (
    Math.max(
      brandMatchScore(context.brandname, developerNames),
      brandMatchScore(context.brandnameRuby, developerNames),
    ) +
    (parseJapaneseScreenshots(result).length > 0 ? 12 : 0)
  );
};

const selectBestVndbResult = (
  results: VndbApiResult[],
  context?: VndbMatchContext,
) => {
  if (!context) return results[0] ?? null;
  if (results.length === 0) return null;

  const scoredByTitle = results.map((result) => ({
    result,
    titleScore: titleScoreForResult(result, context),
  }));
  const bestTitleScore = Math.max(
    ...scoredByTitle.map((scored) => scored.titleScore),
  );
  const nameMatchedResults = scoredByTitle
    .filter((scored) => scored.titleScore === bestTitleScore)
    .map((scored) => scored.result);

  const scoredByDate = nameMatchedResults.map((result) => ({
    result,
    dateScore: releaseDateScore(result.released, context.sellday),
  }));
  const bestDateScore = Math.max(
    ...scoredByDate.map((scored) => scored.dateScore),
  );
  const dateMatchedResults = scoredByDate
    .filter((scored) => scored.dateScore === bestDateScore)
    .map((scored) => scored.result);

  return dateMatchedResults.reduce<VndbApiResult | null>((best, current) => {
    if (!best) return current;
    return tieBreakScoreForResult(current, context) >
      tieBreakScoreForResult(best, context)
      ? current
      : best;
  }, null);
};

export const parseVndbScreenshots = (
  response: VndbApiResponse,
  context?: VndbMatchContext,
): {
  vndbId: string | null;
  matchedTitle: string | null;
  screenshots: VndbScreenshot[];
  status: VndbScreenshotCache["status"];
} => {
  const best = selectBestVndbResult(response.results ?? [], context);
  if (!best) {
    return {
      vndbId: null,
      matchedTitle: null,
      screenshots: [],
      status: "not_found",
    };
  }

  const screenshots = parseJapaneseScreenshots(best);

  return {
    vndbId: best.id,
    matchedTitle: best.alttitle ?? best.title,
    screenshots,
    status: screenshots.length > 0 ? "ok" : "not_found",
  };
};

export const filterVndbScreenshots = (
  screenshots: VndbScreenshot[],
  showSensitive: boolean,
) => {
  if (showSensitive) return screenshots;
  return screenshots.filter(
    (s) =>
      s.sexual < SENSITIVE_THRESHOLD && s.violence < SENSITIVE_THRESHOLD,
  );
};

export const isFreshVndbCache = (
  cache: VndbScreenshotCache | null,
  now = new Date(),
) => {
  if (!cache || cache.status === "error") return false;
  if (!hasCurrentCacheSchema(cache)) return false;
  const fetchedAt = new Date(cache.fetchedAt.replace(" ", "T"));
  if (Number.isNaN(fetchedAt.getTime())) return false;
  return now.getTime() - fetchedAt.getTime() < CACHE_TTL_MS;
};

const hasCurrentCacheSchema = (cache: VndbScreenshotCache) => {
  try {
    const parsed = JSON.parse(cache.screenshotsJson) as {
      version?: number;
      screenshots?: unknown;
    };
    return (
      parsed.version === CACHE_SCHEMA_VERSION &&
      Array.isArray(parsed.screenshots)
    );
  } catch {
    return false;
  }
};

const stringifyScreenshotsCache = (screenshots: VndbScreenshot[]) =>
  JSON.stringify({
    version: CACHE_SCHEMA_VERSION,
    screenshots,
  });

export const readScreenshotsFromCache = (cache: VndbScreenshotCache | null) => {
  if (!cache || cache.status !== "ok") return [];
  try {
    const parsed = JSON.parse(cache.screenshotsJson) as
      | VndbScreenshot[]
      | { screenshots?: VndbScreenshot[] };
    if (Array.isArray(parsed)) return parsed;
    return Array.isArray(parsed.screenshots) ? parsed.screenshots : [];
  } catch {
    return [];
  }
};

export const createVndbRequestBody = (
  collectionElement: Pick<CollectionElement, "gamename"> | string,
) => ({
  filters: [
    "and",
    [
      "search",
      "=",
      typeof collectionElement === "string"
        ? collectionElement
        : collectionElement.gamename,
    ],
    ["has_screenshot", "=", 1],
  ],
  fields:
    "title, alttitle, titles{lang,title,latin,official,main}, aliases, released, developers{name,original,aliases}, screenshots{id,url,thumbnail,dims,thumbnail_dims,sexual,violence,release.languages{lang,mtl}}",
  sort: "searchrank",
  results: 10,
});

const resolveFanzaImageUrl = (url: string) => {
  try {
    return new URL(url, EROGAMESCAPE_REQUEST_PATH + "/").href;
  } catch {
    return url;
  }
};

const toFanzaFullImageUrl = (url: string) =>
  url.replace(/js-(\d+\.(?:jpg|jpeg|png|webp))$/i, "jp-$1");

const isFanzaImageUrl = (url: string) =>
  /^https:\/\/pics\.dmm\.(?:co\.jp|com)\//i.test(url);

const isDlsiteImageUrl = (url: string) =>
  /^https:\/\/img\.dlsite\.jp\//i.test(url);

const isSteamImageUrl = (url: string) =>
  /^https:\/\/[^/]*steamstatic\.com\//i.test(url);

const resolveExternalImageUrl = (url: string) => {
  if (url.startsWith("//")) return `https:${url}`;
  return resolveFanzaImageUrl(url);
};

const getFanzaSampleImageElements = (doc: Document) => {
  const sampleImages = [
    ...doc.querySelectorAll<HTMLImageElement>("#dmm_sample_cg_main img"),
  ];
  if (sampleImages.length > 0) return sampleImages;

  return [...doc.querySelectorAll<HTMLImageElement>("#left_dmm_img img")].filter(
    (image) => {
      const src = resolveFanzaImageUrl(image.getAttribute("src") ?? image.src);
      return isFanzaImageUrl(src) && /js-\d+\.(?:jpg|jpeg|png|webp)$/i.test(src);
    },
  );
};

const extractFanzaProductPageUrl = (doc: Document) => {
  const links = [...doc.querySelectorAll<HTMLAnchorElement>("a[href]")];
  for (const link of links) {
    const href = link.getAttribute("href") ?? "";
    const directMatch = href.match(
      /^https:\/\/dlsoft\.dmm\.(?:co\.jp|com)\/detail\/[^/?#]+\/?/i,
    );
    if (directMatch) {
      return directMatch[0].replace("dlsoft.dmm.co.jp", "dlsoft.dmm.com");
    }

    try {
      const url = new URL(href);
      const lurl = url.searchParams.get("lurl");
      if (!lurl) continue;
      const affiliateMatch = lurl.match(
        /^https:\/\/dlsoft\.dmm\.(?:co\.jp|com)\/detail\/[^/?#]+\/?/i,
      );
      if (affiliateMatch) {
        return affiliateMatch[0].replace("dlsoft.dmm.co.jp", "dlsoft.dmm.com");
      }
    } catch {
      // Ignore non-URL links.
    }
  }
  return null;
};

const extractDlsiteProductPageUrl = (doc: Document) => {
  const links = [...doc.querySelectorAll<HTMLAnchorElement>("a[href]")];
  for (const link of links) {
    const href = link.getAttribute("href") ?? "";
    const match = href.match(
      /^https?:\/\/www\.dlsite\.com\/(home|soft)\/(?:work|dlaf)\/=.*?\/(?:product_id|id)\/([A-Z]{2}\d+)\.html/i,
    );
    if (match) {
      return `https://www.dlsite.com/${match[1]}/work/=/product_id/${match[2]}.html`;
    }
  }
  return null;
};

const extractSteamProductPageUrl = (doc: Document) => {
  const links = [...doc.querySelectorAll<HTMLAnchorElement>("a[href]")];
  for (const link of links) {
    const href = link.getAttribute("href") ?? "";
    const match = href.match(
      /^https:\/\/store\.steampowered\.com\/app\/\d+\/?[^?#]*/i,
    );
    if (match) return match[0];
  }
  return null;
};

const getDmmProductSampleImageUrls = (doc: Document) => {
  const urls = [
    ...doc.querySelectorAll<HTMLImageElement>("img"),
  ].map((image) => resolveFanzaImageUrl(image.getAttribute("src") ?? image.src));
  return urls.filter(
    (url) =>
      isFanzaImageUrl(url) && /\/[^/]+js-\d+\.(?:jpg|jpeg|png|webp)$/i.test(url),
  );
};

const getDlsiteSampleImageUrls = (doc: Document) => {
  const images = [
    ...doc.querySelectorAll<HTMLImageElement>(
      "[id^='dlsite_sample_cg'][id$='_main'] img, img",
    ),
  ];
  return images
    .map((image) => resolveExternalImageUrl(image.getAttribute("src") ?? image.src))
    .filter(
      (url) =>
        isDlsiteImageUrl(url) &&
        /\/[^/]+_img_smp[a-z]?\d+(?:_\d+x\d+)?\.(?:jpg|jpeg|png|webp)$/i.test(
          url,
        ),
    );
};

const getSteamSampleImageUrls = (html: string) => {
  const normalizedHtml = html
    .replaceAll("\\/", "/")
    .replaceAll("&quot;", '"')
    .replaceAll("&amp;", "&");
  const urls = [
    ...normalizedHtml.matchAll(
      /https:\/\/[^"'<>\\]+steamstatic\.com\/[^"'<>\\]+ss_[a-f0-9]+\.\d+x\d+\.jpg(?:\?[^"'<>\\]+)?/gi,
    ),
  ].map((match) => match[0]);
  return urls.filter(isSteamImageUrl);
};

const toFanzaScreenshots = (urls: string[]) => {
  const seen = new Set<string>();
  return urls
    .map((url, index): VndbScreenshot | null => {
      const thumbnail = resolveFanzaImageUrl(url);
      if (!thumbnail || seen.has(thumbnail) || !isFanzaImageUrl(thumbnail)) {
        return null;
      }
      seen.add(thumbnail);
      return {
        id: `fanza-${index + 1}-${thumbnail}`,
        url: toFanzaFullImageUrl(thumbnail),
        thumbnail,
        dims: null,
        thumbnailDims: null,
        sexual: 0,
        violence: 0,
        languages: ["ja"],
      };
    })
    .filter((s): s is VndbScreenshot => !!s);
};

const toDlsiteFullImageUrl = (url: string) =>
  url
    .replace("/resize/images2/", "/modpub/images2/")
    .replace(/_(?:\d+x\d+)(\.(?:jpg|jpeg|png|webp))$/i, "$1");

const toDlsiteScreenshots = (urls: string[]) => {
  const seen = new Set<string>();
  return urls
    .map((url, index): VndbScreenshot | null => {
      const thumbnail = resolveExternalImageUrl(url);
      if (!thumbnail || !isDlsiteImageUrl(thumbnail)) return null;
      const fullUrl = toDlsiteFullImageUrl(thumbnail);
      if (seen.has(fullUrl)) return null;
      seen.add(fullUrl);
      return {
        id: `dlsite-${index + 1}-${fullUrl}`,
        url: fullUrl,
        thumbnail,
        dims: null,
        thumbnailDims: null,
        sexual: 0,
        violence: 0,
        languages: ["ja"],
      };
    })
    .filter((s): s is VndbScreenshot => !!s);
};

const toSteamScreenshots = (urls: string[]) => {
  const byHash = new Map<string, string[]>();
  for (const url of urls) {
    const normalizedUrl = url.replace(/\\\//g, "/");
    const match = normalizedUrl.match(/(ss_[a-f0-9]+)\.(\d+x\d+)\.jpg/i);
    if (!match || !isSteamImageUrl(normalizedUrl)) continue;
    const key = match[1];
    byHash.set(key, [...(byHash.get(key) ?? []), normalizedUrl]);
  }

  return [...byHash.entries()].map(([hash, candidates], index) => {
    const full =
      candidates.find((url) => url.includes(`${hash}.1920x1080.jpg`)) ??
      candidates[candidates.length - 1];
    const thumbnail =
      candidates.find((url) => url.includes(`${hash}.600x338.jpg`)) ??
      candidates.find((url) => url.includes(`${hash}.116x65.jpg`)) ??
      full;
    return {
      id: `steam-${index + 1}-${hash}`,
      url: full,
      thumbnail,
      dims: null,
      thumbnailDims: null,
      sexual: 0,
      violence: 0,
      languages: ["ja"],
    };
  });
};

export const parseFanzaScreenshotsFromHtml = (
  html: string,
): {
  matchedTitle: string | null;
  screenshots: VndbScreenshot[];
  productPageUrl: string | null;
  dlsiteProductPageUrl: string | null;
  steamProductPageUrl: string | null;
} => {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const matchedTitle =
    doc.getElementById("game_title")?.getElementsByTagName("a")[0]?.textContent?.trim() ??
    null;
  const imageElements = getFanzaSampleImageElements(doc);
  const imageUrls = imageElements.map((image) =>
    resolveFanzaImageUrl(image.getAttribute("src") ?? image.src),
  );
  const screenshots = toFanzaScreenshots(imageUrls);

  return {
    matchedTitle,
    screenshots:
      screenshots.length > 0
        ? screenshots
        : toDlsiteScreenshots(getDlsiteSampleImageUrls(doc)),
    productPageUrl: extractFanzaProductPageUrl(doc),
    dlsiteProductPageUrl: extractDlsiteProductPageUrl(doc),
    steamProductPageUrl: extractSteamProductPageUrl(doc),
  };
};

export const parseFanzaScreenshotsFromProductHtml = (
  html: string,
): VndbScreenshot[] => {
  const doc = new DOMParser().parseFromString(html, "text/html");
  return toFanzaScreenshots(getDmmProductSampleImageUrls(doc));
};

export const parseDlsiteScreenshotsFromProductHtml = (
  html: string,
): VndbScreenshot[] => {
  const doc = new DOMParser().parseFromString(html, "text/html");
  return toDlsiteScreenshots(getDlsiteSampleImageUrls(doc));
};

export const parseSteamScreenshotsFromProductHtml = (
  html: string,
): VndbScreenshot[] => toSteamScreenshots(getSteamSampleImageUrls(html));

const getShowSensitiveSetting = async () => {
  const value = await commandGetAppSetting(SHOW_SENSITIVE_SETTING_KEY);
  return value === "true";
};

const getCache = async (collectionElementId: number): Promise<CacheLookup> => {
  const cache = await commandGetVndbScreenshotCache(collectionElementId);
  return { cache, isFresh: isFreshVndbCache(cache) };
};

const requestFanzaScreenshots = async (
  collectionElement: CollectionElement,
): Promise<VndbScreenshotCache> => {
  const response = await fetch(
    `${EROGAMESCAPE_REQUEST_PATH}/game.php?game=${collectionElement.id}`,
    { method: "GET" },
  );
  if (!response.ok) {
    throw new Error(`FANZA screenshot source request failed: ${response.status}`);
  }

  const parsed = parseFanzaScreenshotsFromHtml(await response.text());
  let screenshots = parsed.screenshots;
  if (parsed.productPageUrl) {
    try {
      const productResponse = await fetch(parsed.productPageUrl, { method: "GET" });
      if (productResponse.ok) {
        const productScreenshots = parseFanzaScreenshotsFromProductHtml(
          await productResponse.text(),
        );
        if (productScreenshots.length > screenshots.length) {
          screenshots = productScreenshots;
        }
      }
    } catch (error) {
      console.warn("[fanza] failed to fetch product screenshots", error);
    }
  }
  if (!parsed.productPageUrl && parsed.dlsiteProductPageUrl) {
    try {
      const productResponse = await fetch(parsed.dlsiteProductPageUrl, {
        method: "GET",
      });
      if (productResponse.ok) {
        const productScreenshots = parseDlsiteScreenshotsFromProductHtml(
          await productResponse.text(),
        );
        if (productScreenshots.length > screenshots.length) {
          screenshots = productScreenshots;
        }
      }
    } catch (error) {
      console.warn("[dlsite] failed to fetch product screenshots", error);
    }
  }
  if (screenshots.length === 0 && parsed.steamProductPageUrl) {
    try {
      const productResponse = await fetch(parsed.steamProductPageUrl, {
        method: "GET",
      });
      if (productResponse.ok) {
        screenshots = parseSteamScreenshotsFromProductHtml(
          await productResponse.text(),
        );
      }
    } catch (error) {
      console.warn("[steam] failed to fetch product screenshots", error);
    }
  }
  const cache: VndbScreenshotCache = {
    collectionElementId: collectionElement.id,
    vndbId: null,
    matchedTitle: parsed.matchedTitle ?? collectionElement.gamename,
    screenshotsJson: stringifyScreenshotsCache(screenshots),
    fetchedAt: new Date().toISOString(),
    status: screenshots.length > 0 ? "ok" : "not_found",
  };
  await commandUpsertVndbScreenshotCache(cache);
  return cache;
};

export const ensureVndbScreenshotCache = async (
  collectionElement: CollectionElement,
) => {
  const { cache, isFresh } = await getCache(collectionElement.id);
  if (cache && isFresh) return cache;

  if (inFlight.has(collectionElement.id)) {
    return inFlight.get(collectionElement.id)!;
  }

  const promise = requestFanzaScreenshots(collectionElement)
    .catch(async (error) => {
      console.warn("[fanza] failed to fetch screenshots", error);
      const errorCache: VndbScreenshotCache = {
        collectionElementId: collectionElement.id,
        vndbId: cache?.vndbId ?? null,
        matchedTitle: cache?.matchedTitle ?? null,
        screenshotsJson: cache?.screenshotsJson ?? "[]",
        fetchedAt: new Date().toISOString(),
        status: "error",
      };
      await commandUpsertVndbScreenshotCache(errorCache);
      return errorCache;
    })
    .finally(() => inFlight.delete(collectionElement.id));

  inFlight.set(collectionElement.id, promise);
  return promise;
};

export const loadVndbPreviewScreenshots = async (
  collectionElement: CollectionElement,
) => {
  const showSensitive = await getShowSensitiveSetting();
  const cache = await ensureVndbScreenshotCache(collectionElement);
  return filterVndbScreenshots(readScreenshotsFromCache(cache), showSensitive);
};

const scheduleNextQueueItem = () => {
  if (queueTimer || queue.length === 0) return;
  queueTimer = setTimeout(async () => {
    queueTimer = null;
    const next = queue.shift();
    if (!next) return;
    queuedIds.delete(next.id);
    try {
      const { cache, isFresh } = await getCache(next.id);
      if (!cache || !isFresh) {
        await ensureVndbScreenshotCache(next);
      }
    } finally {
      scheduleNextQueueItem();
    }
  }, PREFETCH_INTERVAL_MS);
};

export const enqueueVndbScreenshotPrefetch = (
  collectionElements: CollectionElement[],
) => {
  for (const element of collectionElements) {
    if (queuedIds.has(element.id) || inFlight.has(element.id)) continue;
    queuedIds.add(element.id);
    queue.push(element);
  }
  scheduleNextQueueItem();
};

export const SHOW_SENSITIVE_VNDB_SCREENSHOTS_SETTING_KEY =
  SHOW_SENSITIVE_SETTING_KEY;
