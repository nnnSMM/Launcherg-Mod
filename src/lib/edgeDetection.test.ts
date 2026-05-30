import { describe, it, expect } from "vitest";
import {
    isNearTopEdge,
    isNearBottomEdge,
    isInsideTopEdgeArea,
} from "./edgeDetection";

describe("edgeDetection - 画面端のカーソル検知", () => {
    describe("isNearTopEdge", () => {
        it("clientYが閾値（デフォルト5px）未満の場合はtrueを返すこと", () => {
            expect(isNearTopEdge(0)).toBe(true);
            expect(isNearTopEdge(2)).toBe(true);
            expect(isNearTopEdge(4)).toBe(true);
        });

        it("clientYが閾値（デフォルト5px）以上の場合はfalseを返すこと", () => {
            expect(isNearTopEdge(5)).toBe(false);
            expect(isNearTopEdge(50)).toBe(false);
        });

        it("カスタム閾値が正しく機能すること", () => {
            expect(isNearTopEdge(29, 30)).toBe(true);
            expect(isNearTopEdge(30, 30)).toBe(false);
        });
    });

    describe("isNearBottomEdge", () => {
        const windowHeight = 1000;

        it("clientYが下端の閾値（デフォルト5px）内にある場合はtrueを返すこと", () => {
            expect(isNearBottomEdge(996, windowHeight)).toBe(true);
            expect(isNearBottomEdge(1000, windowHeight)).toBe(true);
            expect(isNearBottomEdge(995, windowHeight)).toBe(true);
        });

        it("clientYが下端の閾値（デフォルト5px）より外にある場合はfalseを返すこと", () => {
            expect(isNearBottomEdge(994, windowHeight)).toBe(false);
            expect(isNearBottomEdge(500, windowHeight)).toBe(false);
        });

        it("カスタム閾値が正しく機能すること", () => {
            expect(isNearBottomEdge(970, windowHeight, 30)).toBe(true);
            expect(isNearBottomEdge(971, windowHeight, 30)).toBe(true);
            expect(isNearBottomEdge(969, windowHeight, 30)).toBe(false);
        });
    });

    describe("isInsideTopEdgeArea", () => {
        it("指定した上部領域内だけtrueを返すこと", () => {
            expect(isInsideTopEdgeArea(0, 104)).toBe(true);
            expect(isInsideTopEdgeArea(103, 104)).toBe(true);
            expect(isInsideTopEdgeArea(104, 104)).toBe(false);
            expect(isInsideTopEdgeArea(160, 104)).toBe(false);
        });
    });
});
