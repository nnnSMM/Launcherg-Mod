import { describe, expect, it } from "vitest";
import { getGameCandidatesByFilePath } from "@/mock/demoGameMatching";

describe("demo game matching", () => {
  it("matches a real game from its parent folder when the exe is a shared engine", () => {
    const [candidate] = getGameCandidatesByFilePath(
      "E:\\VisualNovel\\key\\Summer Pockets REFLECTION BLUE\\SiglusEngine.exe",
      0.8,
      1,
    );

    expect(candidate?.id).toBe(29016);
  });

  it("matches a real game from a Japanese parent folder when the exe name is generic", () => {
    const [candidate] = getGameCandidatesByFilePath(
      "E:\\VisualNovel\\枕\\サクラノ詩\\BGI.exe",
      0.8,
      1,
    );

    expect(candidate?.id).toBe(4529);
  });

  it("filters installer-like files before automatic linking", () => {
    const candidates = getGameCandidatesByFilePath(
      "E:\\VisualNovel\\枕\\サクラノ詩\\Uninstaller.exe",
      0.8,
      1,
    );

    expect(candidates).toEqual([]);
  });
});
