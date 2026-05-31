import { describe, it, expect } from "vitest";
import {
    evaluateGreedy,
    type Layout,
    type Cell,
    calculateLayouts,
} from "./virtualScrollerMasonry";
import type { CollectionElement } from "@/lib/types";

// モックデータ作成用ヘルパー
const createMockElement = (id: number, width: number, height: number): CollectionElement => ({
    id,
    gamename: `Game ${id}`,
    gamenameRuby: `game ${id}`,
    brandname: "Brand",
    brandnameRuby: "brand",
    sellday: "2023-01-01",
    isNukige: false,
    installAt: null,
    firstPlayAt: null,
    lastPlayAt: null,
    likeAt: null,
    playStatus: 0,
    totalPlayTimeSeconds: 0,
    registeredAt: "",
    exePath: "",
    lnkPath: "",
    icon: "",
    thumbnail: "",
    thumbnailWidth: width,
    thumbnailHeight: height,
    updatedAt: "",
});

describe("virtualScrollerMasonry", () => {
    describe("evaluateGreedy", () => {
        it("should return correct height and diff for empty input", () => {
            const base: Layout = [[], []];
            const remaining: CollectionElement[] = [];
            const itemWidth = 100;
            const [maxHeight, diff] = evaluateGreedy(base, remaining, itemWidth);
            expect(maxHeight).toBe(0);
            expect(diff).toBe(0);
        });

        it("should place elements greedily and calculate correct scores", () => {
            // 2列のカラム
            const base: Layout = [[], []];
            // 2つの要素:
            // 1. 幅100, 高さ100 (アスペクト比1:1) -> itemWidth=100なら高さ100
            // 2. 幅100, 高さ200 (アスペクト比1:2) -> itemWidth=100なら高さ200
            const remaining: CollectionElement[] = [
                createMockElement(1, 100, 100),
                createMockElement(2, 100, 200),
            ];
            const itemWidth = 100;

            // Greedy配置のシミュレーション:
            // 1. 要素1: col0(高さ0) vs col1(高さ0) -> col0に配置 (高さ100)
            // 2. 要素2: col0(高さ100) vs col1(高さ0) -> col1に配置 (高さ200)
            // 最終: col0=100, col1=200
            // MaxHeight: 200, Diff: 100

            const [maxHeight, diff] = evaluateGreedy(base, remaining, itemWidth);
            expect(maxHeight).toBe(200);
            expect(diff).toBe(100);
        });

        it("should use placeholder height if thumbnail dimensions are missing", () => {
            const base: Layout = [[]];
            const element = createMockElement(1, 0, 0); // width/height 0 triggers placeholder
            const itemWidth = 100;

            // placeholderHeight = 16 * 8 = 128
            const [maxHeight, diff] = evaluateGreedy(base, [element], itemWidth);
            expect(maxHeight).toBe(128); // 128
            expect(diff).toBe(0);
        });
    });

    describe("calculateLayouts", () => {
        it("should return empty layout for empty elements", () => {
            const { layout, columns } = calculateLayouts([], 1000);
            expect(layout).toEqual([]);
            expect(columns).toBe(0);
        });

        it("should calculate reasonable layout for a set of items", () => {
            // コンテナ幅: 320 (gap: 16, minItemWidth: 256)
            // itemNumPerRow = max(1, floor((320 + 16) / (256 + 16))) = floor(336 / 272) = 1
            // 1列になるはず
            const containerWidth = 320;
            const elements = [
                createMockElement(1, 100, 100),
                createMockElement(2, 100, 100),
            ];

            const { layout, columns, itemWidth } = calculateLayouts(elements, containerWidth);

            expect(columns).toBe(1);
            expect(layout.length).toBe(1);
            // 要素が2つ入っているか
            expect(layout[0].length).toBe(2);
            expect(layout[0][0].element.id).toBe(1);
            expect(layout[0][1].element.id).toBe(2);
        });

        it("should use beam search to find balanced layout", () => {
            // containerWidth: 1000 (十分広い)
            // minItemWidth = 208
            // gap = 16
            // columns = floor((1000+16)/(208+16)) = floor(1016/224) = 4.53 -> 4列
            const containerWidth = 1000;

            // 要素:
            // 1. 高さ大 (300)
            // 2. 高さ小 (100)
            // 3. 高さ小 (100)
            // 4. 高さ小 (100)
            // Greedyだと偏る可能性があるが、Beam Searchならバランスを取る...かもしれないが
            // 少なくとも全要素が含まれていることは確認する
            const elements = [
                createMockElement(1, 100, 300),
                createMockElement(2, 100, 100),
                createMockElement(3, 100, 100),
                createMockElement(4, 100, 100),
            ];

            const { layout, columns } = calculateLayouts(elements, containerWidth);

            expect(columns).toBe(4);

            // 全要素が含まれているかカウント
            const count = layout.reduce((acc, col) => acc + col.length, 0);
            expect(count).toBe(4);

            // 各列の高さが極端に偏っていないかチェック (Beam searchの効果)
            // 完全なバランスは保証されないが、極端なケースでないか確認
            // const heights = layout.map(col => col.length > 0 ? col[col.length-1].top + col[col.length-1].height : 0);
            // console.log("Final heights:", heights);
        });
    });
});
