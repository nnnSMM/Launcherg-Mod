import { describe, expect, it } from "vitest";
import {
  buildPlayHeatmap,
  calculateDominantColorFromImageData,
  formatLocalDateKey,
  heatmapColorForLevel,
  parseLocalDateKey,
} from "./playHeatmap";

describe("playHeatmap", () => {
  it("builds a Sunday-based GitHub-style 53 week grid", () => {
    const heatmap = buildPlayHeatmap(
      [
        { playDate: "2026-05-24", playTimeSeconds: 3600 },
        { playDate: "2026-05-30", playTimeSeconds: 7200 },
      ],
      new Date(2026, 4, 30),
    );

    expect(heatmap.days).toHaveLength(371);
    expect(heatmap.weekCount).toBe(53);
    expect(heatmap.days[0].weekday).toBe(0);
    expect(heatmap.days.at(-1)?.date).toBe("2026-05-30");
    expect(heatmap.totalSeconds).toBe(10800);
    expect(heatmap.activeDays).toBe(2);
    expect(heatmap.activeWeeks).toBe(1);
    expect(heatmap.longestStreakDays).toBe(1);
  });

  it("counts active weeks and longest consecutive active days", () => {
    const heatmap = buildPlayHeatmap(
      [
        { playDate: "2026-05-20", playTimeSeconds: 1 },
        { playDate: "2026-05-21", playTimeSeconds: 1 },
        { playDate: "2026-05-23", playTimeSeconds: 1 },
        { playDate: "2026-05-25", playTimeSeconds: 1 },
        { playDate: "2026-05-26", playTimeSeconds: 1 },
        { playDate: "2026-05-27", playTimeSeconds: 1 },
      ],
      new Date(2026, 4, 30),
    );

    expect(heatmap.activeDays).toBe(6);
    expect(heatmap.activeWeeks).toBe(2);
    expect(heatmap.longestStreakDays).toBe(3);
  });

  it("aggregates duplicate date rows and scales levels by the max day", () => {
    const heatmap = buildPlayHeatmap(
      [
        { playDate: "2026-05-29", playTimeSeconds: 1800 },
        { playDate: "2026-05-29", playTimeSeconds: 1800 },
        { playDate: "2026-05-30", playTimeSeconds: 7200 },
      ],
      new Date(2026, 4, 30),
    );
    const lower = heatmap.days.find((day) => day.date === "2026-05-29");
    const higher = heatmap.days.find((day) => day.date === "2026-05-30");

    expect(lower?.seconds).toBe(3600);
    expect(lower?.level).toBe(3);
    expect(higher?.level).toBe(5);
  });

  it("parses and formats local date keys without UTC shifting", () => {
    const date = parseLocalDateKey("2026-05-30");

    expect(date).toEqual(new Date(2026, 4, 30));
    expect(formatLocalDateKey(date!)).toBe("2026-05-30");
  });

  it("calculates block-median dominant color from RGBA image data", () => {
    // ピクセル1: R=3, G=4, B=0 -> HSL変換でhue=165度 (ブロック3), weight=0.5 (暗いため)
    // ピクセル2: R=0, G=0, B=12 -> HSL変換でhue=240度 (ブロック5), weight=0.5 (暗いため)
    // 累積重み=1.0, half=0.5。累積が0.5以上になる最初のブロックはブロック3。
    // したがって、選ばれたブロック3のピクセル [3, 4, 0] の平均（そのまま [3, 4, 0]）となるはず。
    const color = calculateDominantColorFromImageData([
      3, 4, 0, 255,
      0, 0, 12, 255,
    ]);

    expect(color).toEqual({ r: 3, g: 4, b: 0 });
  });

  it("reduces influence of white and black pixels by half", () => {
    // 鮮やかな赤(weight=1.0) と純白(weight=0.5) を混ぜると
    // 中央値ブロックが赤側になり r が g,b より大きくなる
    const vividRed  = [200, 10, 10, 255]; // lightness ≈ 0.41 → weight 1.0
    const pureWhite = [255, 255, 255, 255]; // lightness = 1.0  → weight 0.5
    const color = calculateDominantColorFromImageData([
      ...vividRed, ...pureWhite,
    ]);

    expect(color).not.toBeNull();
    expect(color!.r).toBeGreaterThan(color!.b);
    expect(color!.r).toBeGreaterThan(color!.g);
  });

  it("uses the thumbnail dominant color for active heatmap levels", () => {
    expect(heatmapColorForLevel({ r: 10, g: 20, b: 30 }, 4)).toBe(
      "rgba(10, 20, 30, 0.76)",
    );
    expect(heatmapColorForLevel({ r: 10, g: 20, b: 30 }, 0)).toBe(
      "rgba(148, 163, 184, 0.16)",
    );
  });

  it("prevents colors near block boundaries from being split by dynamically shifting block boundaries", () => {
    // R=255, G=178, B=0  -> hue=41.8度 (固定境界45度の直前)
    // R=255, G=182, B=0  -> hue=42.8度 (固定境界45度の直前)
    // R=255, G=195, B=0  -> hue=45.8度 (固定境界45度の直後)
    // 固定境界の場合、これらはブロック0(0~44)とブロック1(45~89)に分断されてしまう。
    // 動的オフセットにより、最もピクセルの多い黄色付近(hue≈43度)がブロックの中心に来るようにシフトされるため、
    // すべて同じブロックに集約され、それらの加重平均が抽出されるはず。
    const color = calculateDominantColorFromImageData([
      255, 178, 0, 255,
      255, 182, 0, 255,
      255, 195, 0, 255,
    ]);

    expect(color).not.toBeNull();
    // 3つのピクセル平均に近い黄色が得られること
    expect(color!.r).toBeCloseTo(255, -1);
    expect(color!.g).toBeGreaterThan(180);
    expect(color!.g).toBeLessThan(190);
    expect(color!.b).toBe(0);
  });
});
