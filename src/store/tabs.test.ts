import { describe, it, expect } from "vitest";
import { isValidTabType } from "./tabs";

describe("tabs", () => {
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
});
