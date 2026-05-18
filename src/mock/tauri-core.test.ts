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

    const preview = await invoke<{
      scannedFileCount: number;
      matchedCount: number;
      results: Array<{ matched: { id: number; gamename: string } | null }>;
    }>("preview_demo_game_matching", {
      exploreDirPaths: [
        "E:\\VisualNovel\\key\\Summer Pockets REFLECTION BLUE",
      ],
    });
    const elements = await invoke<CollectionElement[]>("get_all_elements", {});

    expect(preview.scannedFileCount).toBeGreaterThan(0);
    expect(preview.matchedCount).toBeGreaterThan(0);
    expect(preview.results.some((result) => result.matched?.id === 29016)).toBe(true);
    expect(elements.some((element) => element.id === 29016)).toBe(false);
  });
});
