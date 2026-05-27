import type { Creator, VoiceActor, Work } from "@/lib/types";
import { scrapeSql } from "@/lib/scrapeSql";
import { convertSpecialCharacters } from "@/lib/utils";
import { fetch } from "@tauri-apps/plugin-http";

const BASE_REQUEST_PATH =
  "https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki";
const JAPANESE_TEXT_RE = /[\u3041-\u3096\u30a1-\u30fa\u30fc]/;
const DLSITE_PRODUCT_ID_RE = /(rj|re|vj)\d{4,}/i;
const DLSITE_PRODUCT_ID_GLOBAL_RE = /(rj|re|vj)\d{4,}/gi;

export type ExternalIds = {
  fanzaId?: string;
  dlsiteId?: string;
  dlsiteDomain?: string;
  steamAppId?: string;
  officialUrl?: string;
};

type WorkDescriptionContext = {
  title: string;
  brandName?: string;
  sellday?: string;
  externalIds: ExternalIds;
};

export type DescriptionResult = {
  source: "fanza" | "dlsite" | "steam";
  sourceUrl?: string;
  title?: string;
  brand?: string;
  imageUrl?: string;
  descriptionHtml: string;
};

export const getCreator = (elm: HTMLElement) => {
  const creators: Creator[] = [];
  const aTags = elm.getElementsByTagName("a");
  for (let i = 0; i < aTags.length; i++) {
    const aTag = aTags[i];
    const idStr =
      aTag.getAttribute("href")?.replace("creater.php?creater=", "") ?? "0";
    const id = isNaN(+idStr) ? 0 : +idStr;
    const creator: Creator = {
      id,
      name: convertSpecialCharacters(aTag.innerHTML),
    };
    creators.push(creator);
  }
  return creators;
};
export const getVoiceActors = (elm: HTMLElement) => {
  const creators: VoiceActor[] = [];
  const aTags = elm.getElementsByTagName("a");
  const spanTags = elm.getElementsByTagName("span");
  for (let i = 0; i < aTags.length; i++) {
    const aTag = aTags[i];
    const idStr =
      aTag.getAttribute("href")?.replace("creater.php?creater=", "") ?? "0";
    const id = isNaN(+idStr) ? 0 : +idStr;
    const creator: Creator = {
      id,
      name: convertSpecialCharacters(aTag.innerHTML),
    };
    if (spanTags.length > i) {
      const color = spanTags[i].getAttribute("style");
      const voiceActor: VoiceActor = {
        ...creator,
        role: convertSpecialCharacters(spanTags[i].innerHTML),
        importance: color?.includes("bold")
          ? 0
          : color?.includes("black")
            ? 1
            : 2,
      };
      creators.push(voiceActor);
    }
  }
  return creators;
};
export const getMusics = (elements: HTMLCollectionOf<HTMLTableCellElement>) => {
  const musics: string[] = [];
  for (const elm of elements) {
    const aTag = elm.getElementsByTagName("a")[0];
    if (!aTag) {
      continue;
    }
    if (!aTag.href.includes("music.php?music=")) {
      continue;
    }
    musics.push(aTag.innerHTML);
  }
  return musics;
};

const cleanDescription = (value: string | null | undefined) =>
  convertSpecialCharacters(value ?? "")
    .replace(/\r\n?/g, "\n")
    .replace(/[ \t]+/g, " ")
    .replace(/\n[ \t]+/g, "\n")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();

const uniq = <T>(values: T[]) => [...new Set(values)];

const parseHtml = (html: string) =>
  new DOMParser().parseFromString(html, "text/html");

const normalizeAssetUrl = (value: string, baseUrl?: string) => {
  if (value.startsWith("//")) return `https:${value}`;
  if (!baseUrl || /^[a-z][a-z\d+.-]*:/i.test(value) || value.startsWith("#")) {
    return value;
  }
  try {
    return new URL(value, baseUrl).href;
  } catch {
    return value;
  }
};

type SanitizeDescriptionOptions = {
  removeImages?: boolean;
};

export const sanitizeDescriptionHtml = (
  html: string | null | undefined,
  baseUrl?: string,
  options: SanitizeDescriptionOptions = {}
) => {
  if (!html) return "";
  const doc = parseHtml(`<body>${html}</body>`);

  doc.querySelectorAll("script, style").forEach((elm) => {
    elm.remove();
  });

  doc.querySelectorAll<HTMLElement>("*").forEach((elm) => {
    for (const attr of [...elm.attributes]) {
      if (/^on/i.test(attr.name)) {
        elm.removeAttribute(attr.name);
      }
    }
  });

  doc.querySelectorAll<HTMLAnchorElement>("a[href]").forEach((elm) => {
    const href = elm.getAttribute("href");
    if (href) {
      if (/^\s*javascript:/i.test(href)) {
        elm.removeAttribute("href");
      } else {
        elm.setAttribute("href", normalizeAssetUrl(href, baseUrl));
      }
    }
    elm.setAttribute("target", "_blank");
    elm.setAttribute("rel", "noreferrer");
  });

  if (options.removeImages) {
    doc.querySelectorAll("img").forEach((elm) => {
      elm.remove();
    });
  } else {
    doc.querySelectorAll<HTMLImageElement>("img[src]").forEach((elm) => {
      const src = elm.getAttribute("src");
      if (src) {
        if (/^\s*javascript:/i.test(src)) {
          elm.removeAttribute("src");
        } else {
          elm.setAttribute("src", normalizeAssetUrl(src, baseUrl));
        }
      }
    });
  }

  doc.querySelectorAll<HTMLElement>("[style]").forEach((elm) => {
    const declarations = (elm.getAttribute("style") ?? "")
      .split(";")
      .map((declaration) => declaration.trim())
      .filter(Boolean)
      .filter((declaration) => {
        const property = declaration.split(":")[0]?.trim().toLowerCase();
        return property !== "color" && property !== "background-color";
      });
    if (declarations.length) {
      elm.setAttribute("style", declarations.join("; "));
    } else {
      elm.removeAttribute("style");
    }
  });

  doc.querySelectorAll("font[color]").forEach((elm) => {
    elm.removeAttribute("color");
  });

  return doc.body.innerHTML
    .replace(/\r\n?/g, "\n")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n[ \t]+/g, "\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
};

const htmlToDescriptionText = (html: string | null | undefined) => {
  if (!html) return "";
  const doc = parseHtml(`<body>${html}</body>`);
  doc.querySelectorAll("script, style, img, video, iframe").forEach((elm) => {
    elm.remove();
  });
  doc.querySelectorAll("br").forEach((elm) => {
    elm.replaceWith("\n");
  });
  doc.querySelectorAll("p, div, h1, h2, h3, li").forEach((elm) => {
    elm.append("\n");
  });
  return cleanDescription(doc.body.textContent);
};

const hasJapaneseText = (value: string) => JAPANESE_TEXT_RE.test(value);

const normalizeComparableText = (value: string | null | undefined) =>
  convertSpecialCharacters(value ?? "")
    .normalize("NFKC")
    .toLowerCase()
    .replace(/[【】「」『』（）()\[\]［］"'“”‘’:：・･\-_－−—~〜….,，、。!！?？\s]/g, "");

const levenshteinDistance = (a: string, b: string) => {
  const dp = Array.from({ length: a.length + 1 }, (_, i) =>
    Array.from({ length: b.length + 1 }, (_, j) => (i === 0 ? j : j === 0 ? i : 0))
  );

  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      dp[i][j] = Math.min(
        dp[i - 1][j] + 1,
        dp[i][j - 1] + 1,
        dp[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1)
      );
    }
  }

  return dp[a.length][b.length];
};

export const isTextLikelyRelated = (
  actual: string | null | undefined,
  expected: string | null | undefined,
  threshold = 0.45
) => {
  const actualText = normalizeComparableText(actual);
  const expectedText = normalizeComparableText(expected);
  if (!actualText || !expectedText) return true;
  if (actualText.includes(expectedText) || expectedText.includes(actualText)) {
    return true;
  }
  const longer = Math.max(actualText.length, expectedText.length);
  if (!longer) return true;
  const similarity = 1 - levenshteinDistance(actualText, expectedText) / longer;
  return similarity >= threshold;
};

const hasComparableScript = (
  actual: string | null | undefined,
  expected: string | null | undefined
) => {
  const actualText = convertSpecialCharacters(actual ?? "");
  const expectedText = convertSpecialCharacters(expected ?? "");
  return (
    (/[a-z\d]/i.test(actualText) && /[a-z\d]/i.test(expectedText)) ||
    (JAPANESE_TEXT_RE.test(actualText) && JAPANESE_TEXT_RE.test(expectedText))
  );
};

const isVerifiedDescriptionResult = (
  result: DescriptionResult,
  context: WorkDescriptionContext
) => {
  if (!isTextLikelyRelated(result.title, context.title)) return false;
  if (result.source === "steam") return true;
  if (result.brand && context.brandName && hasComparableScript(result.brand, context.brandName)) {
    return isTextLikelyRelated(result.brand, context.brandName, 0.35);
  }
  return true;
};

export const getWorkDescription = (doc: Document) => {
  const candidates = [
    doc.getElementById("description")?.textContent,
    doc.getElementById("outline")?.textContent,
    doc.getElementById("game_exp")?.textContent,
    doc.querySelector<HTMLMetaElement>('meta[property="og:description"]')
      ?.content,
    doc.querySelector<HTMLMetaElement>('meta[name="description"]')?.content,
  ];

  return candidates
    .map((candidate) => cleanDescription(candidate))
    .find((candidate) => candidate.length > 0);
};

const STORY_START_HEADINGS = [
  "\u30b9\u30c8\u30fc\u30ea\u30fc",
  "\u3042\u3089\u3059\u3058",
  "\u7269\u8a9e",
  "\u7269\u8a9e\u7d39\u4ecb",
  "\u30d7\u30ed\u30ed\u30fc\u30b0",
  "Story",
  "STORY",
  "Introduction",
  "INTRODUCTION",
];

const DLSITE_STORY_START_HEADINGS = [
  ...STORY_START_HEADINGS,
  "\u4f5c\u54c1\u7d39\u4ecb",
  "Concept",
  "CONCEPT",
];

const STORY_END_HEADINGS = [
  "\u30ad\u30e3\u30e9\u30af\u30bf\u30fc",
  "\u30ad\u30e3\u30e9\u30af\u30bf\u30fc\u7d39\u4ecb",
  "\u767b\u5834\u30ad\u30e3\u30e9\u30af\u30bf\u30fc",
  "\u767b\u5834\u30ad\u30e3\u30e9\u30af\u30bf\u30fc\u7d39\u4ecb",
  "\u767b\u5834\u4eba\u7269",
  "\u767b\u5834\u4eba\u7269\u7d39\u4ecb",
  "\u30b7\u30b9\u30c6\u30e0",
  "\u30b2\u30fc\u30e0\u30b7\u30b9\u30c6\u30e0",
  "\u30b2\u30fc\u30e0\u5185\u5bb9",
  "\u30b2\u30fc\u30e0\u7d39\u4ecb",
  "\u30b3\u30f3\u30bb\u30d7\u30c8",
  "Concept",
  "CONCEPT",
  "\u64cd\u4f5c\u65b9\u6cd5",
  "\u30d7\u30ec\u30a4\u65b9\u6cd5",
  "H\u30b7\u30fc\u30f3",
  "\u30a4\u30d9\u30f3\u30c8CG",
  "\u7279\u5178",
  "\u66f4\u65b0\u60c5\u5831",
  "\u88fd\u54c1\u60c5\u5831",
  "\u6ce8\u610f\u4e8b\u9805",
  "\u52d5\u4f5c\u74b0\u5883",
  "\u30b9\u30bf\u30c3\u30d5",
  "STAFF",
  "\u30b5\u30f3\u30d7\u30eb",
  "\u30b5\u30f3\u30d7\u30eb\u753b\u50cf",
  "\u30b9\u30af\u30ea\u30fc\u30f3\u30b7\u30e7\u30c3\u30c8",
  "\u30ae\u30e3\u30e9\u30ea\u30fc",
  "\u4f53\u9a13\u7248",
  "Mature Content Description",
  "System Requirements",
  "Reviews",
  "Feature list",
  "Features",
  "Summary",
  "SUMMARY",
  "Character",
  "CHARACTER",
  "Credit",
  "CREDIT",
  "\u30af\u30ec\u30b8\u30c3\u30c8",
  "DLC",
];

type StoryBlock = {
  html: string;
  text: string;
};

type StoryHeadingMatch = {
  rest: string;
  isHeadingOnly: boolean;
};

type StoryExtractionOptions = {
  startHeadings?: string[];
  requireStartHeading?: boolean;
  allowFallbackFromStart?: boolean;
  removeImages?: boolean;
  baseUrl?: string;
  acceptStartBlock?: (
    block: StoryBlock,
    index: number,
    blocks: StoryBlock[],
    match: StoryHeadingMatch
  ) => boolean;
};

const LONG_SEPARATOR_MIN_LENGTH = 8;
const SEPARATOR_BOUNDARY_RE =
  /^[\s\-_.\u2500\u2501\u2010-\u2015=\uff1d\u2025\u2026\u22ef\u30fb\uff65*＊◆◇■□●○◎★☆※~〜]+$/u;
const STORY_TRAILING_NOTICE_RE =
  /^(?:※\s*)?(?:\u672c\u4f5c|\u672c\u4f5c\u54c1|\u672c\u5546\u54c1|\u672c\u88fd\u54c1|\u3053\u3061\u3089\u3082\u8981\u30c1\u30a7\u30c3\u30af|\u95a2\u9023\u4f5c\u54c1|\u901a\u5e38\u7248|\u5168\u5e74\u9f62\u7248|\u5168\u5e74\u9f62\u5411\u3051|\u65b0\u898f\u653b\u7565\u30d2\u30ed\u30a4\u30f3|\u7d9a\u7de8\u4f5c|\u30a2\u30da\u30f3\u30c9\u30c7\u30a3\u30b9\u30af|\u4e3b\u4eba\u516c\u30d5\u30eb\u30dc\u30a4\u30b9\u5316|\u4e00\u7dd2\u306b\u3084\u308c\u3070\u3088\u308a\u697d\u3057\u3081\u308b|\u524d\u4f5c|.*\u4f5c\u4e2d\u30a4\u30e9\u30b9\u30c8\u62c5\u5f53|\u30b5\u30a6\u30f3\u30c9\u30c8\u30e9\u30c3\u30af|\u30aa\u30ea\u30b8\u30ca\u30eb\u30b5\u30a6\u30f3\u30c9\u30c8\u30e9\u30c3\u30af|\u30a2\u30c3\u30d7\u30c7\u30fc\u30c8|\u66f4\u65b0|ver\.|Ver\.|\(C\)|\uff08C\uff09|©)/i;

const isLongSeparatorBoundaryBlock = (text: string) => {
  const normalizedText = cleanDescription(text).normalize("NFKC");
  const compactText = normalizedText.replace(/\s/g, "");
  return (
    compactText.length >= LONG_SEPARATOR_MIN_LENGTH &&
    SEPARATOR_BOUNDARY_RE.test(compactText)
  );
};

const isStoryEndBoundaryBlock = (text: string) => {
  const normalizedText = cleanDescription(text).normalize("NFKC");
  return (
    isLongSeparatorBoundaryBlock(normalizedText) ||
    STORY_TRAILING_NOTICE_RE.test(normalizedText)
  );
};

const BLOCK_TAGS = new Set([
  "ADDRESS",
  "ARTICLE",
  "ASIDE",
  "BLOCKQUOTE",
  "CENTER",
  "DD",
  "DIV",
  "DL",
  "DT",
  "FIGCAPTION",
  "FIGURE",
  "FOOTER",
  "H1",
  "H2",
  "H3",
  "H4",
  "H5",
  "H6",
  "HEADER",
  "HR",
  "LI",
  "MAIN",
  "NAV",
  "OL",
  "P",
  "PRE",
  "SECTION",
  "TABLE",
  "TBODY",
  "TD",
  "TFOOT",
  "TH",
  "THEAD",
  "TR",
  "UL",
]);

const STORY_BREAK_BLOCK: StoryBlock = { html: "<br>", text: "\n" };
const isStoryBreakBlock = (block: StoryBlock) => block.html === "<br>";

const splitLeafBlockByBreaks = (elm: Element) => {
  const wrapper = document.createElement("div");
  wrapper.innerHTML = elm.innerHTML.replace(/<br\s*\/?>/gi, "<x-story-break></x-story-break>");

  const blocks: StoryBlock[] = [];
  let htmlParts: string[] = [];
  const flush = () => {
    const html = htmlParts.join("").trim();
    htmlParts = [];
    if (!html) return;
    const text = htmlToDescriptionText(html);
    if (text) blocks.push({ html, text });
  };

  for (const child of [...wrapper.childNodes]) {
    if (
      child.nodeType === Node.ELEMENT_NODE &&
      (child as Element).tagName === "X-STORY-BREAK"
    ) {
      flush();
      blocks.push(STORY_BREAK_BLOCK);
      continue;
    }
    if (child.nodeType === Node.TEXT_NODE) {
      htmlParts.push(child.textContent ?? "");
      continue;
    }
    if (child.nodeType === Node.ELEMENT_NODE) {
      htmlParts.push((child as Element).outerHTML);
    }
  }
  flush();
  return blocks;
};

const hasBlockElementChild = (elm: Element) =>
  [...elm.children].some((child) => BLOCK_TAGS.has(child.tagName));

const htmlToStoryBlocks = (html: string) => {
  const doc = parseHtml(`<body>${html}</body>`);
  doc.querySelectorAll("script, style").forEach((elm) => {
    elm.remove();
  });

  const blocks: StoryBlock[] = [];
  const walk = (node: Node) => {
    if (node.nodeType === Node.TEXT_NODE) {
      const text = cleanDescription(node.textContent);
      if (text) blocks.push({ html: text, text });
      return;
    }

    if (node.nodeType !== Node.ELEMENT_NODE) return;
    const elm = node as Element;
    if (["SCRIPT", "STYLE", "IMG", "VIDEO", "IFRAME"].includes(elm.tagName)) {
      return;
    }
    if (elm.tagName === "BR") {
      blocks.push(STORY_BREAK_BLOCK);
      return;
    }

    if (BLOCK_TAGS.has(elm.tagName)) {
      if (hasBlockElementChild(elm)) {
        elm.childNodes.forEach(walk);
        blocks.push(STORY_BREAK_BLOCK);
        return;
      }
      blocks.push(...splitLeafBlockByBreaks(elm));
      blocks.push(STORY_BREAK_BLOCK);
      return;
    }

    if (!hasBlockElementChild(elm)) {
      const text = htmlToDescriptionText(elm.outerHTML);
      if (text) blocks.push({ html: elm.outerHTML, text });
      return;
    }

    elm.childNodes.forEach(walk);
  };

  doc.body.childNodes.forEach(walk);
  return blocks.filter((block) => block.html === "<br>" || cleanDescription(block.text));
};

const trimStoryBreakEdges = (blocks: StoryBlock[]) => {
  let start = 0;
  let end = blocks.length;
  while (start < end && isStoryBreakBlock(blocks[start])) start++;
  while (end > start && isStoryBreakBlock(blocks[end - 1])) end--;
  return blocks.slice(start, end);
};

const normalizeHeadingText = (value: string) =>
  cleanDescription(value)
    .normalize("NFKC")
    .replace(/^[\s【】「」『』\[\]［］（）()<>＜＞◆■□●○★☆*＊#＃\-－ー:：]+/g, "")
    .replace(/[\s【】「」『』\[\]［］（）()<>＜＞◆■□●○★☆*＊#＃\-－ー:：]+$/g, "")
    .trim()
    .toLowerCase();

const isHeadingBlock = (text: string, headings: string[]) => {
  const normalizedText = normalizeHeadingText(text);
  if (!normalizedText || normalizedText.length > 48) return false;
  return headings.some((heading) => {
    const normalizedHeading = normalizeHeadingText(heading);
    return (
      normalizedText === normalizedHeading ||
      normalizedText.startsWith(`${normalizedHeading}:`) ||
      normalizedText.startsWith(`${normalizedHeading}\uff1a`)
    );
  });
};

const STORY_HEADING_DECORATION_START_RE =
  /^[\s"'“”‘’「」『』【】\[\]（）()＜<>〈〉《》◆◇■□●○◎★☆※＊*・:：\-–—―〜~]+/u;
const STORY_HEADING_DECORATION_END_RE =
  /[\s"'“”‘’「」『』【】\[\]（）()＜<>〈〉《》◆◇■□●○◎★☆※＊*・:：\-–—―〜~]+$/u;
const STORY_HEADING_REMAINDER_ALLOWED_START_RE =
  /^[\s"'“”‘’「」『』【】\[\]（）()<>＜＞〈〉《》◆◇■□●○◎★☆※＊*・:：\-–—―〜~\/／]+/u;
const STORY_HEADING_REMAINDER_START_RE =
  /^[\s"'“”‘’」』】\]）)>＞〉》◆◇■□●○◎★☆※＊*・:：\-–—―〜~\/／]+/u;
const INLINE_STORY_HEADING_REST_RE =
  /^[\u3041-\u3096\u30a1-\u30fa\u30fc\u3400-\u9fff]/u;
const INLINE_STORY_HEADING_FALSE_REST_RE =
  /^(?:\u3088\u308a|\u306f|\u304c|\u3092|\u306b|\u306e|\u3082|\u3067\u306f|\u306a\u3089|\u3068\u3057\u3066)/u;

const stripStoryHeadingDecorationStart = (value: string) =>
  value.replace(STORY_HEADING_DECORATION_START_RE, "").trimStart();

const normalizeStoryHeadingText = (value: string) =>
  stripStoryHeadingDecorationStart(cleanDescription(value).normalize("NFKC"))
    .replace(STORY_HEADING_DECORATION_END_RE, "")
    .trim()
    .toLowerCase();

const trimStoryHeadingRemainder = (value: string) =>
  value.replace(STORY_HEADING_REMAINDER_START_RE, "").trim();

const escapeHtml = (value: string) =>
  value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

const findStoryHeadingMatch = (text: string, headings: string[]) => {
  const normalizedText = stripStoryHeadingDecorationStart(
    cleanDescription(text).normalize("NFKC")
  );
  const normalizedLowerText = normalizedText.toLowerCase();
  if (!normalizedLowerText) return null;

  for (const heading of headings) {
    const normalizedHeading = normalizeStoryHeadingText(heading);
    if (!normalizedHeading) continue;
    if (normalizedLowerText === normalizedHeading) {
      return { rest: "", isHeadingOnly: true };
    }
    if (!normalizedLowerText.startsWith(normalizedHeading)) continue;

    const rawRest = normalizedText.slice(normalizedHeading.length);
    const allowsInlineRest =
      (normalizedHeading === normalizeStoryHeadingText("\u30b9\u30c8\u30fc\u30ea\u30fc") ||
        normalizedHeading === normalizeStoryHeadingText("\u3042\u3089\u3059\u3058")) &&
      INLINE_STORY_HEADING_REST_RE.test(rawRest) &&
      !INLINE_STORY_HEADING_FALSE_REST_RE.test(rawRest);
    if (
      !STORY_HEADING_REMAINDER_ALLOWED_START_RE.test(rawRest) &&
      !allowsInlineRest
    ) {
      continue;
    }

    const rest = trimStoryHeadingRemainder(
      rawRest
    );
    if (!rest) {
      return { rest: "", isHeadingOnly: true };
    }
    return { rest, isHeadingOnly: false };
  }

  return null;
};

export const extractStorySectionHtml = (
  html: string,
  options: StoryExtractionOptions = {}
) => {
  const blocks = htmlToStoryBlocks(html);
  const startHeadings = options.startHeadings ?? STORY_START_HEADINGS;
  let startIndex = -1;
  let startMatch: StoryHeadingMatch | null = null;
  for (const [index, block] of blocks.entries()) {
    const match = findStoryHeadingMatch(block.text, startHeadings);
    if (!match) continue;
    if (options.acceptStartBlock?.(block, index, blocks, match) === false) {
      continue;
    }
    startIndex = index;
    startMatch = match;
    break;
  }

  if (startIndex < 0 && options.requireStartHeading) return;

  const collected: StoryBlock[] = [];
  const firstIndex = startIndex >= 0 ? startIndex + 1 : 0;

  if (startIndex < 0 && !options.allowFallbackFromStart) return;
  if (startMatch?.rest) {
    collected.push({ html: escapeHtml(startMatch.rest), text: startMatch.rest });
  }

  for (const block of blocks.slice(firstIndex)) {
    if (findStoryHeadingMatch(block.text, STORY_END_HEADINGS)) break;
    if (collected.length > 0 && isStoryEndBoundaryBlock(block.text)) break;
    collected.push(block);
  }

  const rawHtml = trimStoryBreakEdges(collected)
    .map((block) => block.html)
    .join("");
  const descriptionHtml = sanitizeDescriptionHtml(rawHtml, options.baseUrl, {
    removeImages: options.removeImages ?? true,
  });
  return descriptionHtml && isUsableDescriptionHtml(descriptionHtml)
    ? descriptionHtml
    : undefined;
};

export const extractDlsiteId = (urlOrText: string) =>
  urlOrText.match(DLSITE_PRODUCT_ID_RE)?.[0].toUpperCase() ?? null;

export const extractSteamAppId = (url: string) =>
  url.match(/store\.steampowered\.com\/app\/(\d+)/i)?.[1] ?? null;

export const extractFanzaId = (url: string) => {
  const candidates = [url];
  try {
    candidates.push(decodeURIComponent(url));
  } catch {
    // Ignore malformed percent-encoded strings.
  }

  for (const candidate of candidates) {
    const detailId = candidate.match(
      /dlsoft\.dmm\.(?:co\.jp|com)\/detail\/([^/?#]+)/i
    )?.[1];
    if (detailId) return detailId;

    const cid = candidate.match(/[?&]cid=([^&#/]+)/i)?.[1];
    if (cid) return cid;

    const oldCid = candidate.match(/\/cid=([^/?#]+)\//i)?.[1];
    if (oldCid) return oldCid;
  }
  return null;
};

const getLinkCandidates = (href: string) => {
  const candidates = [href];
  try {
    const url = new URL(href, "https://erogamescape.dyndns.org/");
    if (url.searchParams.get("ch") === "toolbar") return [];
    const lurl = url.searchParams.get("lurl");
    if (lurl) candidates.push(lurl);
  } catch {
    // Relative and malformed links are still checked as raw strings.
  }
  try {
    candidates.push(decodeURIComponent(href));
  } catch {
    // Ignore malformed percent-encoded strings.
  }
  return uniq(candidates);
};

export const extractExternalIdsFromDocument = (doc: Document): ExternalIds => {
  const ids: ExternalIds = {};

  for (const link of [...doc.querySelectorAll<HTMLAnchorElement>("a[href]")]) {
    const href = link.getAttribute("href") ?? "";
    for (const candidate of getLinkCandidates(href)) {
      ids.fanzaId ??= extractFanzaId(candidate) ?? undefined;
      ids.dlsiteId ??= extractDlsiteId(candidate) ?? undefined;
      ids.steamAppId ??= extractSteamAppId(candidate) ?? undefined;
    }
  }

  ids.dlsiteId ??= extractDlsiteId(doc.body?.textContent ?? "") ?? undefined;
  return ids;
};

const normalizeDbValue = (value: string | null | undefined) => {
  const normalized = cleanDescription(value);
  if (
    !normalized ||
    normalized === "0" ||
    normalized.toLowerCase() === "null" ||
    normalized === "-"
  ) {
    return undefined;
  }
  return normalized;
};

const normalizeDlsiteDomain = (value: string | null | undefined) => {
  const normalized = normalizeDbValue(value)?.toLowerCase();
  if (!normalized) return "maniax";
  return /^[a-z0-9_-]+$/.test(normalized) ? normalized : "maniax";
};

const normalizeOfficialUrl = (value: string | null | undefined) => {
  const normalized = normalizeDbValue(value);
  if (!normalized) return undefined;
  try {
    const url = new URL(normalized);
    return url.protocol === "https:" || url.protocol === "http:"
      ? url.href
      : undefined;
  } catch {
    return undefined;
  }
};

export const fetchExternalIdsFromGamelist = async (
  id: number
): Promise<ExternalIds> => {
  const rows = await scrapeSql(
    `select dmm, dlsite_id, coalesce(dlsite_domain, ''), steam, shoukai from gamelist where id = ${id};`,
    5
  );
  const row = rows[0];
  if (!row) return {};

  return {
    fanzaId: normalizeDbValue(row[0]),
    dlsiteId: normalizeDbValue(row[1])?.toUpperCase(),
    dlsiteDomain: normalizeDlsiteDomain(row[2]),
    steamAppId: normalizeDbValue(row[3]),
    officialUrl: normalizeOfficialUrl(row[4]),
  };
};

const mergeExternalIds = (
  primary: ExternalIds,
  fallback: ExternalIds
): ExternalIds => ({
  fanzaId: primary.fanzaId ?? fallback.fanzaId,
  dlsiteId: primary.dlsiteId ?? fallback.dlsiteId,
  dlsiteDomain:
    primary.dlsiteId && primary.dlsiteDomain
      ? primary.dlsiteDomain
      : fallback.dlsiteDomain,
  steamAppId: primary.steamAppId ?? fallback.steamAppId,
  officialUrl: primary.officialUrl ?? fallback.officialUrl,
});

const createGuestId = () =>
  `${Math.random().toString(36).slice(2, 10)}${Date.now().toString(36)}`;

const fanzaRequestOptions = () => ({
  method: "GET",
  headers: {
    Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "ja,en;q=0.8",
    Referer: "https://dlsoft.dmm.co.jp/",
    "User-Agent":
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36",
    Cookie: `age_check_done=1; age_check_new_origin=1; ckcy=1; guest_id=${createGuestId()}`,
  },
});

const dlsiteRequestOptions = () => ({
  method: "GET",
  headers: {
    "Accept-Language": "ja,en;q=0.8",
    Cookie: "adultchecked=1; locale=ja",
  },
});

const officialSiteRequestOptions = () => ({
  method: "GET",
  headers: {
    Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "ja,en;q=0.8",
    "User-Agent":
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36",
  },
});

const fetchHtmlDocument = async (
  url: string,
  options?: Parameters<typeof fetch>[1]
) => {
  try {
    const response = await fetch(url, options);
    if (!response.ok) return;
    return parseHtml(await response.text());
  } catch (error) {
    console.warn(`[description] failed to fetch HTML: ${url}`, error);
  }
};

const fetchJson = async <T>(url: string, options?: Parameters<typeof fetch>[1]) => {
  try {
    const response = await fetch(url, options);
    if (!response.ok) return;
    return (await response.json()) as T;
  } catch (error) {
    console.warn(`[description] failed to fetch JSON: ${url}`, error);
  }
};

const isUsableDescriptionHtml = (descriptionHtml: string) => {
  const description = htmlToDescriptionText(descriptionHtml);
  return description.length > 0 && hasJapaneseText(description);
};

export const createFanzaDetailUrl = (productId: string) =>
  `https://dlsoft.dmm.co.jp/detail/${encodeURIComponent(productId)}/`;

export const createFanzaDoujinDetailUrl = (productId: string) =>
  `https://www.dmm.co.jp/dc/doujin/-/detail/=/cid=${encodeURIComponent(
    productId
  )}/`;

const FANZA_PROMOTION_BLOCK_RE =
  /(\u30c0\u30a6\u30f3\u30ed\u30fc\u30c9\u7248|DL\u7248|\u58c1\u7d19|\u30b5\u30a4\u30ba|\u30ec\u30d3\u30e5\u30fc|\u30ad\u30e3\u30f3\u30da\u30fc\u30f3|\u5fdc\u52df|\u30d1\u30c3\u30b1\u30fc\u30b8\u7248|\u8c6a\u83ef\u7248|\u8c6a\u83ef\u9650\u5b9a\u7248|\u9650\u5b9a\u7248|\u7279\u88c5\u7248|\u7279\u5178|\u7279\u5178\u4ed8\u304d\u7248|\u30c7\u30b8\u30bf\u30eb\u7279\u5178|\u8cfc\u5165|\u62bd\u9078|\u30dd\u30b9\u30bf\u30fc|\u63b2\u8f09\u6e08\u307f|\u30a2\u30c0\u30eb\u30c8PC\u30b2\u30fc\u30e0\u30d5\u30ed\u30a2|\u624b\u306b\u53d6\u3063\u3066\u697d\u3057\u3081\u308b|\u3082\u3063\u3068\u697d\u3057\u307f\u305f\u3044\u65b9|\u5185\u5bb9\u306f\u3053\u3061\u3089|\u8a2d\u5b9a\u8cc7\u6599\u96c6|\u8cc7\u6599\u96c6|\u5c0f\u518a\u5b50|\u30b5\u30a6\u30f3\u30c9\u30c8\u30e9\u30c3\u30af|SOUND\s*TRACK|OST|VOCAL\s*BGM|BGM|\u6b4c\u5531\u66f2|\u30d5\u30eb\u30d0\u30fc\u30b8\u30e7\u30f3|\u5b8c\u5168\u53ce\u9332|\u4f5c\u4e2dBGM|\u30c7\u30fc\u30bf\u7248|\u4ed8\u5c5e|\u540c\u68b1|\u30c9\u30e9\u30deCD|\u30dc\u30a4\u30b9\u30c9\u30e9\u30de|\u30dc\u30a4\u30b9\u96c6|\u30df\u30cb\u30c9\u30e9\u30de|\u30b7\u30b9\u30c6\u30e0\u30dc\u30a4\u30b9|FANZA\s*GAMES|FANZA\u9650\u5b9a|\u9650\u5b9a\u30dc\u30a4\u30b9|\u30c9\u30e9\u30de\u306e\u5404\u30bf\u30a4\u30c8\u30eb|\u540c\u68f2\u30a8\u30d4\u30bd\u30fc\u30c9|\u5f8c\u65e5\u8ac7|\u8ffd\u52a0\u30b7\u30ca\u30ea\u30aa|CD\u97f3\u6e90|\u30d1\u30c3\u30c1|\u30a2\u30da\u30f3\u30c9|\u4fee\u6b63|\u4e0d\u5177\u5408|\u914d\u5e03|\u30d0\u30fc\u30b8\u30e7\u30f3\u60c5\u5831|Ver\.|DMM GAMES PLAYER|\u3054\u6ce8\u610f\u304f\u3060\u3055\u3044|\u3053\u3061\u3089\u3082\u8981\u30c1\u30a7\u30c3\u30af|\u30ad\u30e3\u30e9\u30af\u30bf\u30fc\u7d39\u4ecb)/i;
const FANZA_SEPARATOR_BLOCK_RE =
  /^[\s\-_.\u2501\u2500\u2025\u2026\u30fb\u30fb\u2605\u2606\u25cb\u25cf\u25a0\u25a1\u25c6\u25c7]+$/u;
const FANZA_INLINE_END_RE =
  /(\u3082\u3063\u3068\u307f\u308b|\u30dd\u30a4\u30f3\u30c8\s*[:\uff1a]|\u7279\u96c6\s*[:\uff1a]|\u88fd\u54c1\u60c5\u5831|\u52d5\u4f5c\u74b0\u5883|\u6ce8\u610f\u4e8b\u9805)/;

const isFanzaSeparatorBlock = (text: string) =>
  FANZA_SEPARATOR_BLOCK_RE.test(cleanDescription(text).normalize("NFKC"));

const isFanzaPromotionBlock = (text: string) => {
  const normalizedText = cleanDescription(text).normalize("NFKC");
  if (!normalizedText) return false;
  return (
    !/[A-Za-z0-9\u3041-\u3096\u30a1-\u30fa\u30fc\u3400-\u9fff]/.test(
      normalizedText
    ) ||
    FANZA_SEPARATOR_BLOCK_RE.test(normalizedText) ||
    FANZA_PROMOTION_BLOCK_RE.test(normalizedText) ||
    /^\u672c\u7de8\u300c.*\u300d\u306b[\u3001,]?$/.test(normalizedText) ||
    /^[-\d x×.,]+$/i.test(normalizedText) ||
    /^・?\d{3,4}x\d{3,4}$/i.test(normalizedText)
  );
};

const looksLikeFanzaStoryBlock = (text: string) => {
  const normalizedText = cleanDescription(text);
  return (
    normalizedText.length >= 12 &&
    hasJapaneseText(normalizedText) &&
    !isFanzaPromotionBlock(normalizedText)
  );
};

const FANZA_LEAD_STORY_MIN_TEXT_LENGTH = 30;

const hasPreviousFanzaPromotionBeforeBoundary = (
  blocks: StoryBlock[],
  index: number
) => {
  for (let i = index - 1; i >= 0; i--) {
    if (isLongSeparatorBoundaryBlock(blocks[i].text)) return false;
    if (isFanzaPromotionBlock(blocks[i].text)) return true;
  }
  return false;
};

const hasLaterLongSeparator = (blocks: StoryBlock[], index: number) =>
  blocks.slice(index + 1).some((block) => isLongSeparatorBoundaryBlock(block.text));

const isFanzaPromotionPreludeBlock = (blocks: StoryBlock[], index: number) =>
  hasPreviousFanzaPromotionBeforeBoundary(blocks, index) &&
  hasLaterLongSeparator(blocks, index);

const acceptFanzaStoryStartBlock = (
  block: StoryBlock,
  index: number,
  blocks: StoryBlock[]
) => !isFanzaPromotionBlock(block.text) && !isFanzaPromotionPreludeBlock(blocks, index);

const trimFanzaInlineEnd = (block: StoryBlock) => {
  const text = cleanDescription(block.text);
  const endMatch = text.match(FANZA_INLINE_END_RE);
  if (!endMatch || endMatch.index === undefined) return block;
  const trimmedText = cleanDescription(text.slice(0, endMatch.index));
  return trimmedText
    ? { html: escapeHtml(trimmedText), text: trimmedText }
    : null;
};

const splitFanzaStorySegments = (blocks: StoryBlock[]) => {
  const segments: StoryBlock[][] = [];
  let current: StoryBlock[] = [];
  for (const block of blocks) {
    if (isLongSeparatorBoundaryBlock(block.text)) {
      if (current.length) segments.push(current);
      current = [];
      continue;
    }
    current.push(block);
  }
  if (current.length) segments.push(current);
  return segments;
};

const collectFanzaLeadStoryBlocks = (blocks: StoryBlock[]) => {
  const collected: StoryBlock[] = [];
  for (const block of blocks) {
    if (findStoryHeadingMatch(block.text, STORY_END_HEADINGS)) break;
    if (collected.length > 0 && isStoryEndBoundaryBlock(block.text)) break;
    if (isStoryBreakBlock(block)) {
      if (collected.length > 0) collected.push(block);
      continue;
    }
    if (isFanzaPromotionBlock(block.text)) {
      if (collected.length > 0) {
        if (isFanzaSeparatorBlock(block.text)) continue;
        break;
      }
      continue;
    }

    const startMatch = findStoryHeadingMatch(block.text, STORY_START_HEADINGS);
    if (startMatch) {
      if (startMatch.rest) {
        collected.push({
          html: escapeHtml(startMatch.rest),
          text: startMatch.rest,
        });
      }
      continue;
    }

    const trimmedBlock = trimFanzaInlineEnd(block);
    if (!trimmedBlock) {
      if (collected.length > 0) break;
      continue;
    }
    if (looksLikeFanzaStoryBlock(trimmedBlock.text)) {
      collected.push(trimmedBlock);
    }
    if (trimmedBlock !== block) break;
  }
  return collected;
};

const extractFanzaLeadStoryHtml = (html: string, sourceUrl?: string) => {
  const blocks = htmlToStoryBlocks(html);
  const candidates = splitFanzaStorySegments(blocks)
    .map((segment) => {
      const collected = collectFanzaLeadStoryBlocks(segment);
      const rawHtml = trimStoryBreakEdges(collected)
        .map((block) => block.html)
        .join("");
      const descriptionHtml = sanitizeDescriptionHtml(rawHtml, sourceUrl, {
        removeImages: true,
      });
      const text = htmlToDescriptionText(descriptionHtml);
      return {
        descriptionHtml,
        text,
        hasPromotion: segment.some((block) => isFanzaPromotionBlock(block.text)),
      };
    })
    .filter(
      (candidate) =>
        candidate.descriptionHtml &&
        candidate.text.length >=
          (candidate.hasPromotion ? FANZA_LEAD_STORY_MIN_TEXT_LENGTH : 12) &&
        isUsableDescriptionHtml(candidate.descriptionHtml)
    );

  const candidate =
    candidates.find((item) => !item.hasPromotion) ?? candidates[0];
  return candidate?.descriptionHtml;
};

export const getFanzaDescriptionHtmlFromDocument = (
  doc: Document,
  sourceUrl?: string
) => {
  const root = doc.querySelector<HTMLElement>(
    "section.universalSection div.area-detail-read"
  );
  const textOverflowBlocks = root
    ? [...root.querySelectorAll<HTMLElement>(".read-text-area p.text-overflow")]
    : [];
  const html = textOverflowBlocks.length
    ? textOverflowBlocks.map((elm) => elm.outerHTML).join("")
    : root?.innerHTML ?? "";
  return extractStorySectionHtml(html, {
    baseUrl: sourceUrl,
    requireStartHeading: true,
    removeImages: true,
    acceptStartBlock: acceptFanzaStoryStartBlock,
  }) ?? extractFanzaLeadStoryHtml(html, sourceUrl);
};

export const getFanzaDescriptionFromDocument = (doc: Document) => {
  const descriptionHtml = getFanzaDescriptionHtmlFromDocument(doc);
  return descriptionHtml ? htmlToDescriptionText(descriptionHtml) : undefined;
};

const getFanzaBrandFromDocument = (doc: Document) => {
  for (const label of [
    ...doc.querySelectorAll<HTMLElement>(
      "div.contentsDetailTop__table div.contentsDetailTop__tableDataLeft"
    ),
  ]) {
    if (cleanDescription(label.textContent).includes("\u30d6\u30e9\u30f3\u30c9")) {
      const brand = cleanDescription(label.nextElementSibling?.textContent);
      if (brand) return brand;
    }
  }
};

const getFanzaDescriptionResult = (
  doc: Document,
  sourceUrl: string
): DescriptionResult | null => {
  const descriptionHtml = getFanzaDescriptionHtmlFromDocument(doc, sourceUrl);
  if (!descriptionHtml || !isUsableDescriptionHtml(descriptionHtml)) return null;
  return {
    source: "fanza",
    sourceUrl,
    title: cleanDescription(
      doc.querySelector("h1.productTitle__item")?.textContent
    ),
    brand: getFanzaBrandFromDocument(doc),
    descriptionHtml,
  };
};

export const fetchFromFanza = async (
  context: WorkDescriptionContext
): Promise<DescriptionResult | null> => {
  if (!context.externalIds.fanzaId) return null;
  for (const sourceUrl of [
    createFanzaDetailUrl(context.externalIds.fanzaId),
    createFanzaDoujinDetailUrl(context.externalIds.fanzaId),
  ]) {
    const detailDoc = await fetchHtmlDocument(sourceUrl, fanzaRequestOptions());
    if (!detailDoc) continue;
    const result = getFanzaDescriptionResult(detailDoc, sourceUrl);
    if (result && isVerifiedDescriptionResult(result, context)) return result;
  }
  return null;
};

export const extractDlsiteProductIds = (value: string) =>
  uniq(
    [...value.matchAll(DLSITE_PRODUCT_ID_GLOBAL_RE)].map((match) =>
      match[0].toUpperCase()
    )
  );

export const createDlsiteDetailUrl = (
  productId: string,
  dlsiteDomain?: string
) => {
  const normalizedProductId = productId.toUpperCase();
  const service =
    dlsiteDomain === undefined
      ? normalizedProductId.startsWith("VJ")
        ? "pro"
        : "maniax"
      : normalizeDlsiteDomain(dlsiteDomain);
  return `https://www.dlsite.com/${service}/work/=/product_id/${normalizedProductId}.html?locale=ja`;
};

export const createDlsiteAnnounceUrl = (
  productId: string,
  dlsiteDomain?: string
) => {
  const normalizedProductId = productId.toUpperCase();
  const service =
    dlsiteDomain === undefined
      ? normalizedProductId.startsWith("VJ")
        ? "pro"
        : "maniax"
      : normalizeDlsiteDomain(dlsiteDomain);
  return `https://www.dlsite.com/${service}/announce/=/product_id/${normalizedProductId}.html?locale=ja`;
};

export const getDlsiteDescriptionHtmlFromDocument = (
  doc: Document,
  sourceUrl?: string
) => {
  const html =
    doc.querySelector<HTMLElement>('[itemprop="description"]')?.innerHTML ?? "";
  return extractStorySectionHtml(html, {
    baseUrl: sourceUrl,
    startHeadings: DLSITE_STORY_START_HEADINGS,
    requireStartHeading: true,
    removeImages: true,
  });
};

export const getDlsiteDescriptionFromDocument = (doc: Document) => {
  const descriptionHtml = getDlsiteDescriptionHtmlFromDocument(doc);
  return descriptionHtml ? htmlToDescriptionText(descriptionHtml) : undefined;
};

const getDlsiteDescriptionResult = (
  doc: Document,
  sourceUrl: string
): DescriptionResult | null => {
  const descriptionHtml = getDlsiteDescriptionHtmlFromDocument(doc, sourceUrl);
  if (!descriptionHtml || !isUsableDescriptionHtml(descriptionHtml)) return null;
  return {
    source: "dlsite",
    sourceUrl,
    title: cleanDescription(doc.querySelector("#work_name")?.textContent),
    brand: cleanDescription(doc.querySelector(".maker_name a")?.textContent),
    descriptionHtml,
  };
};

const fetchDlsiteDescriptionByProductId = async (
  productId: string,
  dlsiteDomain?: string
) => {
  for (const sourceUrl of [
    createDlsiteDetailUrl(productId, dlsiteDomain),
    createDlsiteAnnounceUrl(productId, dlsiteDomain),
  ]) {
    const detailDoc = await fetchHtmlDocument(sourceUrl, dlsiteRequestOptions());
    if (!detailDoc) continue;
    const result = getDlsiteDescriptionResult(detailDoc, sourceUrl);
    if (result) return result;
  }
  return null;
};

export const fetchFromDlsite = async (
  context: WorkDescriptionContext
): Promise<DescriptionResult | null> => {
  if (!context.externalIds.dlsiteId) return null;
  const result = await fetchDlsiteDescriptionByProductId(
    context.externalIds.dlsiteId,
    context.externalIds.dlsiteDomain
  );
  return result && isVerifiedDescriptionResult(result, context) ? result : null;
};

export const createSteamAppDetailsUrl = (appId: number) => {
  const url = new URL("https://store.steampowered.com/api/appdetails");
  url.searchParams.set("appids", `${appId}`);
  url.searchParams.set("l", "japanese");
  url.searchParams.set("cc", "JP");
  return url.href;
};

type SteamAppDetailsPayload = Record<
  string,
  {
    success?: boolean;
    data?: {
      name?: string;
      detailed_description?: string;
      about_the_game?: string;
      short_description?: string;
      developers?: string[];
      publishers?: string[];
      header_image?: string;
      capsule_image?: string;
      capsule_imagev5?: string;
    };
  }
>;

const looksLikeFeatureDescription = (html: string) => {
  const blocks = htmlToStoryBlocks(html);
  if (!blocks.length) return false;
  const firstText = blocks
    .slice(0, 3)
    .map((block) => block.text)
    .join("\n");
  return /(\u7279\u5fb4|\u6a5f\u80fd|\u30b7\u30b9\u30c6\u30e0|features?|DLC|\u30a2\u30c3\u30d7\u30c7\u30fc\u30c8|\u30d1\u30c3\u30c1)/i.test(
    firstText
  );
};

const extractSteamStoryHtml = (
  html: string | null | undefined,
  sourceUrl: string,
  allowFallbackFromStart: boolean
) => {
  if (!html) return;
  const storyHtml = extractStorySectionHtml(html, {
    baseUrl: sourceUrl,
    allowFallbackFromStart,
    removeImages: true,
  });
  if (!storyHtml) return;
  if (allowFallbackFromStart && looksLikeFeatureDescription(storyHtml)) {
    return;
  }
  return storyHtml;
};

export const getSteamDescriptionHtmlFromAppDetails = (
  payload: SteamAppDetailsPayload,
  appId: number
) => {
  const data = payload[`${appId}`]?.data;
  if (!payload[`${appId}`]?.success || !data) return;
  const sourceUrl = `https://store.steampowered.com/app/${appId}`;
  const shortDescriptionHtml = sanitizeDescriptionHtml(
    data.short_description,
    sourceUrl,
    { removeImages: true }
  );
  return (
    extractSteamStoryHtml(data.detailed_description, sourceUrl, true) ??
    extractSteamStoryHtml(data.about_the_game, sourceUrl, true) ??
    (isUsableDescriptionHtml(shortDescriptionHtml)
      ? shortDescriptionHtml
      : undefined)
  );
};

export const getSteamDescriptionFromAppDetails = (
  payload: SteamAppDetailsPayload,
  appId: number
) => {
  const descriptionHtml = getSteamDescriptionHtmlFromAppDetails(payload, appId);
  return descriptionHtml ? htmlToDescriptionText(descriptionHtml) : undefined;
};

const getSteamDescriptionResult = (
  payload: SteamAppDetailsPayload,
  appId: number
): DescriptionResult | null => {
  const data = payload[`${appId}`]?.data;
  const descriptionHtml = getSteamDescriptionHtmlFromAppDetails(payload, appId);
  if (!data || !descriptionHtml) return null;
  const sourceUrl = `https://store.steampowered.com/app/${appId}`;
  return {
    source: "steam",
    sourceUrl,
    title: cleanDescription(data.name),
    brand: cleanDescription(data.developers?.join(", ")),
    imageUrl: normalizeAssetUrl(
      data.header_image ?? data.capsule_image ?? data.capsule_imagev5 ?? "",
      sourceUrl
    ),
    descriptionHtml,
  };
};

export const findSteamAppIdFromOfficialDocument = (doc: Document) => {
  const ids = extractExternalIdsFromDocument(doc);
  if (ids.steamAppId) return ids.steamAppId;

  const html = doc.documentElement?.innerHTML ?? "";
  return extractSteamAppId(html) ?? undefined;
};

export const fetchSteamAppIdFromOfficialSite = async (officialUrl?: string) => {
  if (!officialUrl) return;
  const officialDoc = await fetchHtmlDocument(
    officialUrl,
    officialSiteRequestOptions()
  );
  return officialDoc ? findSteamAppIdFromOfficialDocument(officialDoc) : undefined;
};

export const fetchFromSteam = async (
  context: WorkDescriptionContext
): Promise<DescriptionResult | null> => {
  const steamAppId =
    context.externalIds.steamAppId ??
    (await fetchSteamAppIdFromOfficialSite(context.externalIds.officialUrl));
  const appId = Number(steamAppId);
  if (!Number.isFinite(appId) || appId <= 0) return null;
  const payload = await fetchJson<SteamAppDetailsPayload>(
    createSteamAppDetailsUrl(appId),
    { method: "GET" }
  );
  if (!payload) return null;
  const result = getSteamDescriptionResult(payload, appId);
  return result && isVerifiedDescriptionResult(result, context) ? result : null;
};

const fetchDescriptionSafely = async (
  source: DescriptionResult["source"],
  fetcher: () => Promise<DescriptionResult | null>
) => {
  try {
    return await fetcher();
  } catch (error) {
    console.warn(`[${source}] failed to fetch description`, error);
    return null;
  }
};

export const fetchJapaneseDescriptionResult = async (
  context: WorkDescriptionContext
) => {
  for (const [source, fetcher] of [
    ["fanza", () => fetchFromFanza(context)],
    ["dlsite", () => fetchFromDlsite(context)],
    ["steam", () => fetchFromSteam(context)],
  ] as const) {
    const result = await fetchDescriptionSafely(source, fetcher);
    if (result) return result;
  }
  return null;
};

export const fetchJapaneseDescription = async (context: WorkDescriptionContext) => {
  const result = await fetchJapaneseDescriptionResult(context);
  return result ? htmlToDescriptionText(result.descriptionHtml) : undefined;
};

export const getWorkByScrape = async (id: number) => {
  const [response, gamelistExternalIds] = await Promise.all([
    fetch(`${BASE_REQUEST_PATH}/game.php?game=${id}`, {
      method: "GET",
    }),
    fetchExternalIdsFromGamelist(id),
  ]);
  const parser = new DOMParser();
  const doc = parser.parseFromString(await response.text(), "text/html");

  const gameTitle = doc.getElementById("game_title");
  const softTitle = doc.getElementById("soft-title");
  const illustrators = doc
    .getElementById("genga")
    ?.getElementsByTagName("td")[0];
  const writers = doc.getElementById("shinario")?.getElementsByTagName("td")[0];
  const voiceActors = doc
    .getElementById("seiyu")
    ?.getElementsByTagName("td")[0];

  const musics = doc
    .getElementById("music_summary_main")
    ?.getElementsByTagName("td");
  const name = convertSpecialCharacters(
    gameTitle?.getElementsByTagName("a")[0].innerHTML ?? ""
  );
  const brandName = convertSpecialCharacters(
    softTitle?.getElementsByTagName("a")[0].innerHTML ?? ""
  );
  const sellday = softTitle?.getElementsByTagName("a")[1].innerHTML ?? "2030-01-01";
  const externalIds = mergeExternalIds(
    gamelistExternalIds,
    extractExternalIdsFromDocument(doc)
  );
  const descriptionResult = await fetchJapaneseDescriptionResult({
    title: name,
    brandName,
    sellday,
    externalIds: {
      ...externalIds,
    },
  });
  const description = descriptionResult
    ? htmlToDescriptionText(descriptionResult.descriptionHtml)
    : undefined;
  const erogeScapeImageUrl =
    doc.getElementById("main_image")?.getElementsByTagName("img")[0].src ?? "";

  const work: Work = {
    id: id,
    name,
    brandId: +(
      softTitle
        ?.getElementsByTagName("a")[0]
        ?.getAttribute("href")
        ?.replace("brand.php?brand=", "") ?? "0"
    ),
    brandName,
    description,
    sellday,
    imgUrl: erogeScapeImageUrl || descriptionResult?.imageUrl || "",
    officialHomePage:
      gameTitle?.getElementsByTagName("a")[0].getAttribute("href") ?? "",
    statistics: {
      median: +(
        doc.getElementById("median")?.getElementsByTagName("td")[0].innerHTML ??
        "0"
      ),
      count: +(
        doc.getElementById("count")?.getElementsByTagName("td")[0].innerHTML ??
        "0"
      ),
      average: +(
        doc.getElementById("average")?.getElementsByTagName("td")[0]
          .innerHTML ?? "0"
      ),
      playTime:
        doc.getElementById("play_time")?.getElementsByTagName("td")[0]
          .innerHTML ?? "0時間",
    },
    creators: {
      illustrators: illustrators ? getCreator(illustrators) : [],
      writers: writers ? getCreator(writers) : [],
      voiceActors: voiceActors ? getVoiceActors(voiceActors) : [],
    },
    musics: musics ? getMusics(musics) : [],
  };
  return work;
};
