import { describe, it, expect, vi } from "vitest";
import { parseQueryResultHtml } from "./scrapeSql";

// convertSpecialCharacters のモック
vi.mock("@/lib/utils", () => ({
    convertSpecialCharacters: (str: string) => str.replace(/&amp;/g, "&"),
}));

describe("scrapeSql", () => {
    describe("parseQueryResultHtml", () => {
        it("should parse table rows from HTML", () => {
            const html = `
        <table id="query_result_main">
          <tr><th>ID</th><th>Name</th></tr>
          <tr><td>1</td><td>Game A</td></tr>
          <tr><td>2</td><td>Game B</td></tr>
        </table>
      `;

            const result = parseQueryResultHtml(html, 2);

            expect(result).toHaveLength(2);
            expect(result[0]).toEqual(["1", "Game A"]);
            expect(result[1]).toEqual(["2", "Game B"]);
        });

        it("should skip first row (header)", () => {
            const html = `
        <table id="query_result_main">
          <tr><th>Header1</th><th>Header2</th></tr>
          <tr><td>Data1</td><td>Data2</td></tr>
        </table>
      `;

            const result = parseQueryResultHtml(html, 2);

            expect(result).toHaveLength(1);
            expect(result[0]).toEqual(["Data1", "Data2"]);
        });

        it("should skip rows with missing columns", () => {
            const html = `
        <table id="query_result_main">
          <tr><th>A</th><th>B</th><th>C</th></tr>
          <tr><td>1</td><td>2</td><td>3</td></tr>
          <tr><td>4</td></tr>
        </table>
      `;

            // 3列を期待するが、2行目は1列しかないのでスキップ
            const result = parseQueryResultHtml(html, 3);

            expect(result).toHaveLength(1);
            expect(result[0]).toEqual(["1", "2", "3"]);
        });

        it("should return empty array for empty table", () => {
            const html = `<table id="query_result_main"></table>`;

            const result = parseQueryResultHtml(html, 2);

            expect(result).toEqual([]);
        });

        it("should return empty array when table not found", () => {
            const html = `<div>No table here</div>`;

            const result = parseQueryResultHtml(html, 2);

            expect(result).toEqual([]);
        });

        it("should handle special characters via convertSpecialCharacters", () => {
            const html = `
        <table id="query_result_main">
          <tr><th>Name</th></tr>
          <tr><td>Game &amp; More</td></tr>
        </table>
      `;

            const result = parseQueryResultHtml(html, 1);

            expect(result).toHaveLength(1);
            expect(result[0][0]).toBe("Game & More");
        });
    });
});
