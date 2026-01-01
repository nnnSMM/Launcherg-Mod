import { describe, it, expect } from "vitest";
import { mapRowToElementDetail } from "./registerCollectionElementDetails";

describe("mapRowToElementDetail", () => {
    it("should correctly map a row to element details", () => {
        // row: [id, gamenameRuby, sellday, okazu, brandname, brandfurigana]
        // okazu: 't' means true (nukige), other false
        const row = ["12345", "test game ruby", "2023-10-27", "t", "Test Brand", "test brand ruby"];

        const result = mapRowToElementDetail(row);

        expect(result).toEqual({
            collectionElementId: 12345,
            gamenameRuby: "test game ruby",
            sellday: "2023-10-27",
            isNukige: true,
            brandname: "Test Brand",
            brandnameRuby: "test brand ruby",
        });
    });

    it("should handle non-nukige flag correctly", () => {
        // 'f' or empty string -> false
        const row = ["123", "ruby", "2023-01-01", "f", "Brand", "Ruby"];
        const result = mapRowToElementDetail(row);
        expect(result.isNukige).toBe(false);

        const rowEmpty = ["124", "ruby", "2023-01-01", "", "Brand", "Ruby"];
        const resultEmpty = mapRowToElementDetail(rowEmpty);
        expect(resultEmpty.isNukige).toBe(false);
    });

    it("should handle numeric string conversion for ID", () => {
        const row = ["007", "ruby", "2023-01-01", "t", "Brand", "Ruby"];
        const result = mapRowToElementDetail(row);
        expect(result.collectionElementId).toBe(7);
    });
});
