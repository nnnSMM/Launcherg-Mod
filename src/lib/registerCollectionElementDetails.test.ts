import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  commandCreateElementDetails,
  commandGetNotRegisterdDetailElementIds,
} from "@/lib/command";
import { scrapeSql } from "@/lib/scrapeSql";
import {
  __resetRegisterCollectionElementDetailsForTest,
  mapRowToElementDetail,
  registerCollectionElementDetails,
} from "./registerCollectionElementDetails";

vi.mock("@/lib/command", () => ({
  commandCreateElementDetails: vi.fn(),
  commandGetNotRegisterdDetailElementIds: vi.fn(),
}));

vi.mock("@/lib/scrapeSql", () => ({
  scrapeSql: vi.fn(),
}));

describe("registerCollectionElementDetails", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    __resetRegisterCollectionElementDetailsForTest();
    vi.mocked(commandGetNotRegisterdDetailElementIds).mockResolvedValue([]);
    vi.mocked(scrapeSql).mockResolvedValue([]);
    vi.mocked(commandCreateElementDetails).mockResolvedValue(undefined);
  });

  it("maps rows to element details", () => {
    const row = [
      "12345",
      "test game ruby",
      "2023-10-27",
      "t",
      "Test Brand",
      "test brand ruby",
    ];

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

  it("handles non-nukige flag correctly", () => {
    const row = ["123", "ruby", "2023-01-01", "f", "Brand", "Ruby"];
    const result = mapRowToElementDetail(row);
    expect(result.isNukige).toBe(false);

    const rowEmpty = ["124", "ruby", "2023-01-01", "", "Brand", "Ruby"];
    const resultEmpty = mapRowToElementDetail(rowEmpty);
    expect(resultEmpty.isNukige).toBe(false);
  });

  it("handles numeric string conversion for ID", () => {
    const row = ["007", "ruby", "2023-01-01", "t", "Brand", "Ruby"];
    const result = mapRowToElementDetail(row);
    expect(result.collectionElementId).toBe(7);
  });

  it("deduplicates concurrent registration runs", async () => {
    vi.mocked(commandGetNotRegisterdDetailElementIds).mockResolvedValue([
      123,
      124,
    ]);
    vi.mocked(scrapeSql).mockResolvedValue([
      ["123", "ruby1", "2023-01-01", "t", "Brand1", "BrandRuby1"],
      ["124", "ruby2", "2023-01-02", "f", "Brand2", "BrandRuby2"],
    ]);

    await Promise.all([
      registerCollectionElementDetails(),
      registerCollectionElementDetails(),
      registerCollectionElementDetails(),
    ]);

    expect(commandGetNotRegisterdDetailElementIds).toHaveBeenCalledTimes(1);
    expect(scrapeSql).toHaveBeenCalledTimes(1);
    expect(scrapeSql).toHaveBeenCalledWith(
      expect.stringContaining("123, 124"),
      6,
    );
    expect(commandCreateElementDetails).toHaveBeenCalledTimes(1);
    expect(commandCreateElementDetails).toHaveBeenCalledWith([
      {
        collectionElementId: 123,
        gamenameRuby: "ruby1",
        sellday: "2023-01-01",
        isNukige: true,
        brandname: "Brand1",
        brandnameRuby: "BrandRuby1",
      },
      {
        collectionElementId: 124,
        gamenameRuby: "ruby2",
        sellday: "2023-01-02",
        isNukige: false,
        brandname: "Brand2",
        brandnameRuby: "BrandRuby2",
      },
    ]);
  });

  it("does not scrape when there are no missing detail IDs", async () => {
    await registerCollectionElementDetails();

    expect(commandGetNotRegisterdDetailElementIds).toHaveBeenCalledTimes(1);
    expect(scrapeSql).not.toHaveBeenCalled();
    expect(commandCreateElementDetails).not.toHaveBeenCalled();
  });

  it("allows a later retry after a failed run", async () => {
    vi.mocked(commandGetNotRegisterdDetailElementIds)
      .mockRejectedValueOnce(new Error("load failed"))
      .mockResolvedValueOnce([]);

    await expect(registerCollectionElementDetails()).rejects.toThrow(
      "load failed",
    );
    await registerCollectionElementDetails();

    expect(commandGetNotRegisterdDetailElementIds).toHaveBeenCalledTimes(2);
  });
});
