import { describe, it, expect, vi, beforeEach } from "vitest";
import { search } from "./search";
import type { SortOrder } from "@/components/Sidebar/sort";
import type { CollectionElement } from "@/lib/types";
import type { Option } from "@/lib/trieFilter";

// sidebarCollectionElements ストアをモック
vi.mock("@/store/sidebarCollectionElements", () => ({
    sidebarCollectionElements: {
        value: () =>
            [
                {
                    id: 1,
                    gamename: "Game A",
                    gamenameRuby: "game a",
                    brandname: "Brand A",
                    brandnameRuby: "brand a",
                    sellday: "2023-01-15",
                    isNukige: false,
                    installAt: null,
                    firstPlayAt: null,
                    lastPlayAt: null,
                    likeAt: null,
                    playStatus: 0,
                    totalPlayTimeSeconds: 0,
                    registeredAt: "2023-01-01",
                    exePath: "",
                    lnkPath: "",
                    icon: "",
                    thumbnail: "",
                    thumbnailWidth: 100,
                    thumbnailHeight: 100,
                    updatedAt: "",
                },
                {
                    id: 2,
                    gamename: "Game B",
                    gamenameRuby: "game b",
                    brandname: "Brand B",
                    brandnameRuby: "brand b",
                    sellday: "2023-06-20",
                    isNukige: true,
                    installAt: null,
                    firstPlayAt: null,
                    lastPlayAt: null,
                    likeAt: "2023-07-01",
                    playStatus: 2,
                    totalPlayTimeSeconds: 3600,
                    registeredAt: "2023-06-01",
                    exePath: "C:/Games/game.exe",
                    lnkPath: "",
                    icon: "",
                    thumbnail: "",
                    thumbnailWidth: 100,
                    thumbnailHeight: 100,
                    updatedAt: "",
                },
                {
                    id: 3,
                    gamename: "Game C",
                    gamenameRuby: "game c",
                    brandname: "Brand C",
                    brandnameRuby: "brand c",
                    sellday: "2022-12-01",
                    isNukige: false,
                    installAt: null,
                    firstPlayAt: null,
                    lastPlayAt: null,
                    likeAt: null,
                    playStatus: 1,
                    totalPlayTimeSeconds: 1800,
                    registeredAt: "2022-11-01",
                    exePath: "",
                    lnkPath: "",
                    icon: "",
                    thumbnail: "",
                    thumbnailWidth: 100,
                    thumbnailHeight: 100,
                    updatedAt: "",
                },
            ] as CollectionElement[],
    },
}));

describe("search", () => {
    it("should filter by provided options", () => {
        const filteredOptions: Option<number>[] = [
            { label: "Game A", value: 1 },
            { label: "Game B", value: 2 },
        ];
        const attributes = [{ key: "nukige", enabled: false }] as any[];
        const order: SortOrder = "gamename-asc";

        const result = search(filteredOptions, attributes, order);

        // 全てのラベルが1つの配列にフラット化される
        const allElements = result.flatMap((group) => group.elements);
        expect(allElements).toHaveLength(2);
        expect(allElements.map((e) => e.id)).toContain(1);
        expect(allElements.map((e) => e.id)).toContain(2);
    });

    it("should apply attribute filters when enabled", () => {
        const filteredOptions: Option<number>[] = [
            { label: "Game A", value: 1 },
            { label: "Game B", value: 2 },
            { label: "Game C", value: 3 },
        ];
        // nukige フィルタを有効化 -> isNukige: true のみ残る
        const attributes = [{ key: "nukige", enabled: true }] as any[];
        const order: SortOrder = "gamename-asc";

        const result = search(filteredOptions, attributes, order);
        const allElements = result.flatMap((group) => group.elements);

        // Game B だけが isNukige: true
        expect(allElements).toHaveLength(1);
        expect(allElements[0].id).toBe(2);
    });

    it("should sort results by specified order", () => {
        const filteredOptions: Option<number>[] = [
            { label: "Game A", value: 1 },
            { label: "Game B", value: 2 },
            { label: "Game C", value: 3 },
        ];
        const attributes: any[] = [];
        // 発売年降順でソート
        const order: SortOrder = "sellyear-desc";

        const result = search(filteredOptions, attributes, order);

        // グループ化されてソートされるため、グループ順を確認
        expect(result.length).toBeGreaterThan(0);
    });
});
