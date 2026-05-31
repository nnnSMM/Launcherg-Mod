import { describe, expect, it } from "vitest";
import type { CollectionElementDailyPlayTime } from "@/lib/types";
import { mergeDailyPlayTimes } from "@/lib/playHeatmapHelper";

describe("mergeDailyPlayTimes", () => {
  it("should return an empty array if given an empty array", () => {
    const result = mergeDailyPlayTimes([]);
    expect(result).toEqual([]);
  });

  it("should merge play times for the same date and sort by date ascending", () => {
    const input: CollectionElementDailyPlayTime[] = [
      { collectionElementId: 1, playDate: "2026-05-20", playTimeSeconds: 3000 },
      { collectionElementId: 2, playDate: "2026-05-20", playTimeSeconds: 4000 },
      { collectionElementId: 1, playDate: "2026-05-18", playTimeSeconds: 2000 },
      { collectionElementId: 3, playDate: "2026-05-21", playTimeSeconds: 1500 },
    ];

    const expected: CollectionElementDailyPlayTime[] = [
      { collectionElementId: 0, playDate: "2026-05-18", playTimeSeconds: 2000 },
      { collectionElementId: 0, playDate: "2026-05-20", playTimeSeconds: 7000 },
      { collectionElementId: 0, playDate: "2026-05-21", playTimeSeconds: 1500 },
    ];

    const result = mergeDailyPlayTimes(input);
    expect(result).toEqual(expected);
  });

  it("should handle single item arrays correctly", () => {
    const input: CollectionElementDailyPlayTime[] = [
      { collectionElementId: 123, playDate: "2026-05-19", playTimeSeconds: 5000 },
    ];
    const expected: CollectionElementDailyPlayTime[] = [
      { collectionElementId: 0, playDate: "2026-05-19", playTimeSeconds: 5000 },
    ];
    const result = mergeDailyPlayTimes(input);
    expect(result).toEqual(expected);
  });
});
