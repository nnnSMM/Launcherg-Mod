export type PlayHeatmapSource = {
  playDate: string;
  playTimeSeconds: number;
};

export type RgbColor = {
  r: number;
  g: number;
  b: number;
};

export type PlayHeatmapDay = {
  date: string;
  seconds: number;
  level: number;
  weekIndex: number;
  weekday: number;
  isToday: boolean;
  isFuture: boolean;
};

export type PlayHeatmapMonth = {
  label: string;
  weekIndex: number;
};

export type PlayHeatmap = {
  days: PlayHeatmapDay[];
  months: PlayHeatmapMonth[];
  weekCount: number;
  totalSeconds: number;
  activeDays: number;
  maxDailySeconds: number;
};

const DEFAULT_WEEK_COUNT = 53;
const DAYS_IN_WEEK = 7;

export const fallbackHeatmapColor: RgbColor = {
  r: 96,
  g: 165,
  b: 250,
};

const startOfLocalDay = (date: Date) =>
  new Date(date.getFullYear(), date.getMonth(), date.getDate());

const addDays = (date: Date, days: number) => {
  const next = new Date(date);
  next.setDate(next.getDate() + days);
  return next;
};

export const formatLocalDateKey = (date: Date) => {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
};

export const parseLocalDateKey = (value: string): Date | null => {
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
  if (!match) return null;

  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) {
    return null;
  }

  return new Date(year, month - 1, day);
};

export const buildPlayHeatmap = (
  source: PlayHeatmapSource[],
  today = new Date(),
  weekCount = DEFAULT_WEEK_COUNT,
): PlayHeatmap => {
  const normalizedToday = startOfLocalDay(today);
  const currentWeekStart = addDays(normalizedToday, -normalizedToday.getDay());
  const firstDate = addDays(currentWeekStart, -(weekCount - 1) * DAYS_IN_WEEK);
  const secondsByDate = new Map<string, number>();

  for (const row of source) {
    const date = parseLocalDateKey(row.playDate);
    if (!date) continue;

    const key = formatLocalDateKey(date);
    const seconds = Math.max(0, Math.floor(row.playTimeSeconds));
    secondsByDate.set(key, (secondsByDate.get(key) ?? 0) + seconds);
  }

  const rawDays = Array.from({ length: weekCount * DAYS_IN_WEEK }, (_, index) => {
    const date = addDays(firstDate, index);
    const key = formatLocalDateKey(date);
    const isFuture = date.getTime() > normalizedToday.getTime();
    return {
      date: key,
      seconds: isFuture ? 0 : secondsByDate.get(key) ?? 0,
      level: 0,
      weekIndex: Math.floor(index / DAYS_IN_WEEK),
      weekday: date.getDay(),
      isToday: key === formatLocalDateKey(normalizedToday),
      isFuture,
    };
  });

  const maxDailySeconds = rawDays.reduce(
    (max, day) => Math.max(max, day.seconds),
    0,
  );
  const days = rawDays.map((day) => ({
    ...day,
    level:
      day.seconds <= 0 || maxDailySeconds <= 0
        ? 0
        : Math.max(1, Math.ceil((day.seconds / maxDailySeconds) * 5)),
  }));

  const months: PlayHeatmapMonth[] = [];
  let previousMonth = -1;
  for (let weekIndex = 0; weekIndex < weekCount; weekIndex += 1) {
    const weekStart = addDays(firstDate, weekIndex * DAYS_IN_WEEK);
    const month = weekStart.getMonth();
    if (month !== previousMonth) {
      months.push({
        label: `${month + 1}月`,
        weekIndex,
      });
      previousMonth = month;
    }
  }

  return {
    days,
    months,
    weekCount,
    totalSeconds: days.reduce((sum, day) => sum + day.seconds, 0),
    activeDays: days.filter((day) => day.seconds > 0).length,
    maxDailySeconds,
  };
};

const HISTOGRAM_BLOCKS = 8;

export const calculateDominantColorFromImageData = (
  data: ArrayLike<number>,
): RgbColor | null => {
  interface PixelInfo {
    r: number;
    g: number;
    b: number;
    hue: number;
    w: number;
  }
  const pixels: PixelInfo[] = [];

  // 色相の粗い分布を把握するための36ビン（10度刻み）の細粒度ヒストグラム
  const fineHistogram = new Float64Array(36);
  let totalWeight = 0;

  for (let i = 0; i + 3 < data.length; i += 4) {
    const alpha = data[i + 3] / 255;
    if (alpha <= 0) continue;

    const r = data[i]     / 255;
    const g = data[i + 1] / 255;
    const b = data[i + 2] / 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const lightness = (max + min) / 2;
    const chroma = max - min;

    let hue = 0;
    if (chroma !== 0) {
      if (max === r) {
        hue = ((g - b) / chroma) % 6;
      } else if (max === g) {
        hue = (b - r) / chroma + 2;
      } else {
        hue = (r - g) / chroma + 4;
      }
      hue = Math.round(hue * 60);
      if (hue < 0) hue += 360;
    }

    // HSL明度で白・黒ピクセルの影響を半減
    const w = (lightness > 0.8 || lightness < 0.15) ? alpha * 0.5 : alpha;

    pixels.push({ r: data[i], g: data[i + 1], b: data[i + 2], hue, w });

    const fineBin = Math.min(35, Math.floor(hue / 10));
    fineHistogram[fineBin] += w;
    totalWeight += w;
  }

  if (totalWeight <= 0 || pixels.length === 0) return null;

  // 最もピクセルが多く集まっている（重みの大きい）支配的な色相ピーク P を探す
  let maxWeight = -1;
  let maxBin = 0;
  for (let bin = 0; bin < 36; bin++) {
    if (fineHistogram[bin] > maxWeight) {
      maxWeight = fineHistogram[bin];
      maxBin = bin;
    }
  }
  const P = maxBin * 10 + 5; // ピーク色相ビンの中心角

  // ピーク色相 P がブロック0のど真ん中（22.5度付近）に来るように
  // ブロック境界の区切り位置を (P - 22.5) 度シフトする
  const shift = P - 22.5;

  // シフトされた境界に基づいて8ブロックに分類・累積
  const blockSumR = new Float64Array(HISTOGRAM_BLOCKS);
  const blockSumG = new Float64Array(HISTOGRAM_BLOCKS);
  const blockSumB = new Float64Array(HISTOGRAM_BLOCKS);
  const blockWeight = new Float64Array(HISTOGRAM_BLOCKS);

  for (const p of pixels) {
    const shiftedHue = (p.hue - shift + 360) % 360;
    const bin = Math.min(HISTOGRAM_BLOCKS - 1, Math.floor(shiftedHue / 45));

    blockSumR[bin]   += p.r * p.w;
    blockSumG[bin]   += p.g * p.w;
    blockSumB[bin]   += p.b * p.w;
    blockWeight[bin] += p.w;
  }

  const half = totalWeight / 2;
  let cumulative = 0;
  let medianBin = HISTOGRAM_BLOCKS - 1;
  for (let bin = 0; bin < HISTOGRAM_BLOCKS; bin++) {
    cumulative += blockWeight[bin];
    if (cumulative >= half) {
      medianBin = bin;
      break;
    }
  }

  const bw = blockWeight[medianBin];
  const rAvg = bw > 0 ? Math.round(blockSumR[medianBin] / bw) : 0;
  const gAvg = bw > 0 ? Math.round(blockSumG[medianBin] / bw) : 0;
  const bAvg = bw > 0 ? Math.round(blockSumB[medianBin] / bw) : 0;

  return { r: rAvg, g: gAvg, b: bAvg };
};

export const heatmapColorForLevel = (baseColor: RgbColor, level: number) => {
  if (level <= 0) {
    return "rgba(148, 163, 184, 0.16)";
  }

  const alphaByLevel = [0, 0.26, 0.42, 0.58, 0.76, 0.95];
  const alpha = alphaByLevel[Math.min(5, level)] ?? alphaByLevel[5];
  return `rgba(${baseColor.r}, ${baseColor.g}, ${baseColor.b}, ${alpha})`;
};
