import { describe, it, expect, vi, beforeEach } from "vitest";
import { isValidTabType, deleteTab, tabs, selected, setDemoBuildForTest } from "./tabs";
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
});
