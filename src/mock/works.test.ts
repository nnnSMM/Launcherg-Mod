import { describe, expect, it } from "vitest";
import { works } from "./works";

describe("mock works", () => {
  it("should have ensureRegisteredStories method which resolves without errors", async () => {
    expect(works.ensureRegisteredStories).toBeTypeOf("function");
    await expect(works.ensureRegisteredStories()).resolves.toBeUndefined();
  });
});
