import { describe, it, expect } from "vitest";
import { collectionElementsToOptions, useTrieFilter } from "./trieFilter";
import { get } from "svelte/store";
import type { CollectionElement } from "@/lib/types";

// モックデータ作成ヘルパー
const createMockElement = (id: number, name: string, ruby: string, brand: string, brandRuby: string): CollectionElement => ({
    id,
    gamename: name,
    gamenameRuby: ruby,
    brandname: brand,
    brandnameRuby: brandRuby,
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
    thumbnailWidth: 100,
    thumbnailHeight: 100,
    updatedAt: "",
});

describe("trieFilter", () => {
    describe("collectionElementsToOptions", () => {
        it("should convert collection elements to options with search labels", () => {
            const elements = [
                createMockElement(1, "ゲームA", "げーむえー", "ブランドA", "ぶらんどえー"),
            ];
            const options = collectionElementsToOptions(elements);

            expect(options).toHaveLength(1);
            expect(options[0].label).toBe("ゲームA");
            expect(options[0].value).toBe(1);
            // otherLabelsには、ひらがな、ローマ字が含まれるはず
            expect(options[0].otherLabels).toContain("げーむえー");
            expect(options[0].otherLabels).toContain("ge-mue-"); // wanakanaのローマ字変換
        });
    });

    describe("useTrieFilter", () => {
        it("should filter options based on query", () => {
            const elements = [
                createMockElement(1, "Game One", "game one", "Brand A", "brand a"),
                createMockElement(2, "Game Two", "game two", "Brand B", "brand b"),
                createMockElement(3, "Another", "another", "Brand C", "brand c"),
            ];
            const optionsData = collectionElementsToOptions(elements);

            const optionsStore = {
                subscribe: (run: (value: any) => void) => {
                    run(optionsData);
                    return () => { };
                }
            };

            const { query, filtered } = useTrieFilter(
                optionsStore as any,
                () => optionsData
            );

            // 初期状態: 全件表示
            expect(get(filtered)).toHaveLength(3);

            // 検索: "Another" -> このクエリで1件ヒット
            query.set("Another");
            expect(get(filtered).some(opt => opt.value === 3)).toBe(true);
        });

        it("should support hiragana/romaji search", () => {
            const elements = [
                createMockElement(10, "テスト", "てすと", "Brand", "brand"),
            ];
            const optionsData = collectionElementsToOptions(elements);
            const optionsStore = { subscribe: (run: any) => { run(optionsData); return () => { }; } };

            const { query, filtered } = useTrieFilter(optionsStore as any, () => optionsData);

            // ひらがな検索 "てすと"
            query.set("てすと");
            expect(get(filtered)).toHaveLength(1);
            expect(get(filtered)[0].label).toBe("テスト");

            // ローマ字検索 "tesuto"
            query.set("tesuto");
            expect(get(filtered)).toHaveLength(1);
            expect(get(filtered)[0].label).toBe("テスト");
        });
    });
});
