import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CollectionElement } from "@/lib/types";

describe("demo tauri core", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.resetModules();
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: vi.fn(() => "blob:demo-icon"),
    });
  });

  it("adds a matched game from a folder path through the automatic import command", async () => {
    const { invoke } = await import("@/mock/tauri-core");

    const added = await invoke<string[]>("create_elements_in_pc", {
      exploreDirPaths: [
        "E:\\VisualNovel\\key\\Summer Pockets REFLECTION BLUE",
      ],
      useCache: true,
    });
    const elements = await invoke<CollectionElement[]>("get_all_elements", {});
    const imported = elements.find((element) => element.id === 29016);

    expect(added).toEqual(["Summer Pockets REFLECTION BLUE"]);
    expect(imported).toBeTruthy();
    expect(imported?.icon).toBe(imported?.thumbnail);
    expect(imported?.icon).not.toBe("images/dummy_thumbnail.svg");
  });

  it("does not treat a direct file path as an automatic import root", async () => {
    const { invoke } = await import("@/mock/tauri-core");

    const added = await invoke<string[]>("create_elements_in_pc", {
      exploreDirPaths: [
        "E:\\VisualNovel\\key\\Summer Pockets REFLECTION BLUE\\SiglusEngine.exe",
      ],
      useCache: true,
    });

    expect(added).toEqual([]);
  });

  it("uses an icon from the selected real folder when the browser exposes file contents", async () => {
    const makeFile = (content: string, name: string, type = "") =>
      Object.assign(new Blob([content], { type }), { name }) as File;
    const selectedDirectory = {
      kind: "directory",
      name: "Summer Pockets REFLECTION BLUE",
      entries: async function* () {
        const exeHandle = {
          kind: "file",
          name: "SiglusEngine.exe",
          getFile: async () => makeFile("MZ", "SiglusEngine.exe"),
        };
        const icoHandle = {
          kind: "file",
          name: "game.ico",
          getFile: async () => makeFile("ico", "game.ico", "image/x-icon"),
        };
        yield [
          "SiglusEngine.exe",
          exeHandle,
        ];
        yield ["game.ico", icoHandle];
      },
    };
    const demoWindow = window as Window & {
      showDirectoryPicker?: () => Promise<typeof selectedDirectory>;
    };
    demoWindow.showDirectoryPicker = async () => selectedDirectory;

    const { pickDemoDirectory } = await import("@/mock/demoBrowserFiles");
    const selectedPath = await pickDemoDirectory();
    const { invoke } = await import("@/mock/tauri-core");

    const added = await invoke<string[]>("create_elements_in_pc", {
      exploreDirPaths: [selectedPath],
      useCache: true,
    });
    const elements = await invoke<CollectionElement[]>("get_all_elements", {});
    const imported = elements.find((element) => element.id === 29016);

    expect(added).toEqual(["Summer Pockets REFLECTION BLUE"]);
    expect(imported?.icon).toBe("blob:demo-icon");
    expect(imported?.icon).not.toBe(imported?.thumbnail);
  });

  it("previews folder matching without adding games to the collection", async () => {
    const { invoke } = await import("@/mock/tauri-core");
    const { listen } = await import("@/mock/tauri-event");
    let max = 0;
    let processed = 0;
    const unlisten = await listen("progresslive", (event) => {
      const payload = event.payload as { max?: number } | undefined;
      if (typeof payload?.max === "number") {
        max = payload.max;
      } else {
        processed += 1;
      }
    });

    const preview = await invoke<{
      scannedFileCount: number;
      matchedCount: number;
      results: Array<{ path: string; matched: { id: number; gamename: string } | null }>;
    }>("preview_demo_game_matching", {
      exploreDirPaths: [
        "E:\\VisualNovel\\key\\Summer Pockets REFLECTION BLUE",
      ],
    });
    unlisten();
    const elements = await invoke<CollectionElement[]>("get_all_elements", {});

    expect(preview.scannedFileCount).toBeGreaterThan(0);
    expect(max).toBe(preview.scannedFileCount);
    expect(processed).toBe(preview.scannedFileCount);
    expect(preview.matchedCount).toBe(1);
    expect(preview.results.filter((result) => result.matched?.id === 29016)).toHaveLength(1);
    expect(preview.results.find((result) => result.matched?.id === 29016)?.path).toMatch(/BGI\.exe$/);
    expect(elements.some((element) => element.id === 29016)).toBe(false);
  });

  it("hides review-only files in a folder that already has a high-confidence match", async () => {
    const makeFile = (content: string, name: string) =>
      Object.assign(new Blob([content]), { name }) as File;
    const selectedDirectory = {
      kind: "directory",
      name: "nekoneko",
      entries: async function* () {
        const sumireDirectory = {
          kind: "directory",
          name: "すみれ",
          entries: async function* () {
            yield [
              "すみれ.exe",
              {
                kind: "file",
                name: "すみれ.exe",
                getFile: async () => makeFile("MZ", "すみれ.exe"),
              },
            ];
            yield [
              "config.exe",
              {
                kind: "file",
                name: "config.exe",
                getFile: async () => makeFile("MZ", "config.exe"),
              },
            ];
          },
        };
        yield ["すみれ", sumireDirectory];
      },
    };
    const demoWindow = window as Window & {
      showDirectoryPicker?: () => Promise<typeof selectedDirectory>;
    };
    demoWindow.showDirectoryPicker = async () => selectedDirectory;

    const { pickDemoDirectory } = await import("@/mock/demoBrowserFiles");
    const selectedPath = await pickDemoDirectory();
    const { invoke } = await import("@/mock/tauri-core");
    const preview = await invoke<{
      matchedCount: number;
      results: Array<{ path: string; matched: { id: number } | null }>;
    }>("preview_demo_game_matching", {
      exploreDirPaths: selectedPath ? [selectedPath] : [],
    });

    expect(preview.matchedCount).toBe(1);
    expect(preview.results).toHaveLength(1);
    expect(preview.results[0]?.matched?.id).toBe(20178);
    expect(preview.results.some((result) => result.path.endsWith("config.exe"))).toBe(false);
  });
});
