import { describe, it, expect, vi, beforeEach } from "vitest";
import {
    isValidTabType,
    deleteTab,
    tabs,
    selected,
    setDemoBuildForTest,
    syncSelectedToLocation,
    routeLoaded,
} from "./tabs";
import { push, replace } from "svelte-spa-router";
import { get } from "svelte/store";

vi.mock("svelte-spa-router", () => ({
  push: vi.fn(),
  replace: vi.fn(),
}));

describe("tabs", () => {
    beforeEach(() => {
        localStorage.clear();
        tabs.set([]);
        selected.set(-1);
        vi.mocked(push).mockClear();
        vi.mocked(replace).mockClear();
    });

    describe("isValidTabType", () => {
        it("should return true for 'works'", () => {
            expect(isValidTabType("works")).toBe(true);
        });

        it("should return true for 'memos'", () => {
            expect(isValidTabType("memos")).toBe(true);
        });

        it("should return true for 'settings'", () => {
            expect(isValidTabType("settings")).toBe(true);
        });

        it("should return true for 'stats'", () => {
            expect(isValidTabType("stats")).toBe(true);
        });

        it("should return false for invalid type", () => {
            expect(isValidTabType("invalid")).toBe(false);
            expect(isValidTabType("")).toBe(false);
            expect(isValidTabType("work")).toBe(false);
            expect(isValidTabType("setting")).toBe(false);
        });
    });

    describe("deleteTab", () => {
        beforeEach(() => {
            tabs.set([
                { id: 1, type: "works", scrollTo: 0, title: "Game 1", path: "/works/1" },
                { id: 2, type: "works", scrollTo: 0, title: "Game 2", path: "/works/2" }
            ]);
            selected.set(1);
        });

        it("通常ビルド時：タブがすべて削除されると / に遷移する", () => {
            setDemoBuildForTest(false);
            
            deleteTab(2);
            expect(replace).toHaveBeenCalledWith("/works/1");
            
            vi.mocked(replace).mockClear();
            deleteTab(1);
            
            expect(replace).toHaveBeenCalledWith("/");
            expect(get(selected)).toBe(-1);
        });

        it("デモビルド時：タブがすべて削除されると /demo に遷移する", () => {
            setDemoBuildForTest(true);
            
            deleteTab(2);
            expect(replace).toHaveBeenCalledWith("/works/1");
            
            vi.mocked(replace).mockClear();
            deleteTab(1);
            
            expect(replace).toHaveBeenCalledWith("/demo");
            expect(get(selected)).toBe(-1);
        });
    });

    describe("syncSelectedToLocation", () => {
        it("should update the selected tab index from the current route", () => {
            tabs.set([
                { id: 1, type: "works", workId: 10, scrollTo: 0, title: "Game 10", path: "/works/10?gamename=Game%2010" },
                { id: 2, type: "memos", workId: 10, scrollTo: 0, title: "Memo - Game 10", path: "/memos/10?gamename=Game%2010" },
                { id: -101, type: "settings", scrollTo: 0, title: "ショートカット設定", path: "/settings/shortcut" },
                { id: -102, type: "stats", scrollTo: 0, title: "統計", path: "/stats" },
            ]);
            selected.set(0);

            syncSelectedToLocation("/memos/10");
            expect(get(selected)).toBe(1);

            syncSelectedToLocation("/settings/shortcut");
            expect(get(selected)).toBe(2);

            syncSelectedToLocation("/stats");
            expect(get(selected)).toBe(3);

            syncSelectedToLocation("/");
            expect(get(selected)).toBe(-1);
        });
    });

    describe("routeLoaded - 統計タブ", () => {
        it("統計ページを固定タブとして作成すること", () => {
            routeLoaded({
                detail: {
                    location: "/stats",
                    querystring: "",
                },
            } as any);

            const currentTabs = get(tabs);
            expect(currentTabs).toEqual([
                { id: -102, type: "stats", scrollTo: 0, title: "統計", path: "/stats" },
            ]);
            expect(get(selected)).toBe(0);
        });
    });

    describe("routeLoaded - メモタブの自動挿入位置", () => {
        it("対応するゲームタブが存在する場合、そのゲームタブのすぐ右隣に新規メモタブを作成すること", () => {
            tabs.set([
                { id: 101, type: "settings", scrollTo: 0, title: "設定", path: "/settings/shortcut" },
                { id: 10, type: "works", workId: 10, scrollTo: 0, title: "Game 10", path: "/works/10?gamename=Game%2010" },
                { id: 20, type: "works", workId: 20, scrollTo: 0, title: "Game 20", path: "/works/20?gamename=Game%2020" },
            ]);
            selected.set(0);

            routeLoaded({
                detail: {
                    location: "/memos/10",
                    querystring: "gamename=Game%2010",
                },
            } as any);

            const currentTabs = get(tabs);
            expect(currentTabs.length).toBe(4);
            expect(currentTabs[2].type).toBe("memos");
            expect(currentTabs[2].workId).toBe(10);
            expect(get(selected)).toBe(2);
        });
    });
});

