import { describe, it, expect, vi, beforeEach } from "vitest";

// Vitest 実行環境用にグローバル変数をモック
// @ts-ignore
globalThis.__PUBLIC_DEMO_BUILD__ = false;

import { appUpdate } from "./update";
import { get } from "svelte/store";
import { commandGetAppSetting, commandSetAppSetting } from "@/lib/command";
import { check } from "@tauri-apps/plugin-updater";
import { fetch as httpFetch } from "@tauri-apps/plugin-http";

vi.mock("@/lib/command", () => ({
  commandGetAppSetting: vi.fn(),
  commandSetAppSetting: vi.fn(),
}));

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn(() => Promise.resolve("0.3.3")),
}));

vi.mock("@tauri-apps/plugin-http", () => ({
  fetch: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

describe("appUpdate store", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.unstubAllEnvs();
    vi.stubEnv("DEV", false);
    vi.mocked(httpFetch).mockResolvedValue({ ok: false } as any);
    appUpdate.resetForTest();
    localStorage.clear();
    window.history.replaceState({}, "", "/");
  });

  describe("初期状態", () => {
    it("状態が idle で、ダイアログが閉じており、アップデート情報が null であること", () => {
      const state = get(appUpdate);
      expect(state.status).toBe("idle");
      expect(state.update).toBeNull();
      expect(state.isDialogOpen).toBe(false);
      expect(state.installMessage).toBe("");
      expect(state.installProgress).toBeNull();
      expect(state.error).toBeNull();
    });
  });

  describe("ダイアログの表示制御", () => {
    it("openDialog でダイアログが開き、closeDialog でダイアログが閉じること", () => {
      appUpdate.openDialog();
      expect(get(appUpdate).isDialogOpen).toBe(true);

      appUpdate.closeDialog();
      expect(get(appUpdate).isDialogOpen).toBe(false);
    });

    it("toggleDialog でダイアログの開閉が切り替わること", () => {
      expect(get(appUpdate).isDialogOpen).toBe(false);

      appUpdate.toggleDialog();
      expect(get(appUpdate).isDialogOpen).toBe(true);

      appUpdate.toggleDialog();
      expect(get(appUpdate).isDialogOpen).toBe(false);
    });
  });

  describe("初期化とバージョンチェックの振る舞い", () => {
    it("dev 起動では mock 指定がない限り updater check を呼ばないこと", async () => {
      vi.stubEnv("DEV", true);

      await appUpdate.initialize();

      const state = get(appUpdate);
      expect(check).not.toHaveBeenCalled();
      expect(state.status).toBe("none");
      expect(state.update).toBeNull();
    });

    it("dev 起動でも mockUpdate 指定時は mock 更新通知を表示すること", async () => {
      vi.stubEnv("DEV", true);
      window.history.replaceState({}, "", "/?mockUpdate=1");
      vi.mocked(commandGetAppSetting).mockResolvedValue(null);

      await appUpdate.initialize();

      const state = get(appUpdate);
      expect(check).not.toHaveBeenCalled();
      expect(state.status).toBe("available");
      expect(state.update?.isMock).toBe(true);
      expect(state.update?.canInstall).toBe(false);
    });

    it("アップデートがない場合は status が none になること", async () => {
      vi.mocked(check).mockResolvedValue(null);

      await appUpdate.initialize();

      const state = get(appUpdate);
      expect(state.status).toBe("none");
      expect(state.update).toBeNull();
    });

    it("アップデートが存在する場合は status が available になりアップデート情報が設定されること", async () => {
      const mockTauriUpdate = {
        version: "26.5.24",
        currentVersion: "0.3.3",
        date: "2026-05-24T00:00:00Z",
        body: "アップデート内容",
        downloadAndInstall: vi.fn(),
      };
      vi.mocked(check).mockResolvedValue(mockTauriUpdate as any);
      vi.mocked(commandGetAppSetting).mockResolvedValue(null);

      await appUpdate.initialize();

      const state = get(appUpdate);
      expect(state.status).toBe("available");
      expect(state.update).not.toBeNull();
      expect(state.update?.version).toBe("26.5.24");
      expect(state.update?.canInstall).toBe(true);
    });

    it("検出されたアップデートが無視設定されているバージョンの場合は status が ignored になりアップデート情報が null になること", async () => {
      const mockTauriUpdate = {
        version: "26.5.24",
        currentVersion: "0.3.3",
        date: "2026-05-24T00:00:00Z",
        body: "アップデート内容",
        downloadAndInstall: vi.fn(),
      };
      vi.mocked(check).mockResolvedValue(mockTauriUpdate as any);
      vi.mocked(commandGetAppSetting).mockResolvedValue("26.5.24");

      await appUpdate.initialize();

      const state = get(appUpdate);
      expect(state.status).toBe("ignored");
      expect(state.update).toBeNull();
    });
  });

  describe("バージョン無視機能", () => {
    it("アップデート情報が存在しない場合は ignoreCurrentVersion を呼んでも無視リストへの保存が行われないこと", async () => {
      await appUpdate.ignoreCurrentVersion();
      expect(commandSetAppSetting).not.toHaveBeenCalled();
    });

    it("アップデート情報が存在する場合、ignoreCurrentVersion を呼ぶと無視設定が保存され status が ignored になること", async () => {
      const mockTauriUpdate = {
        version: "26.5.24",
        currentVersion: "0.3.3",
        date: "2026-05-24T00:00:00Z",
        body: "アップデート内容",
        downloadAndInstall: vi.fn(),
      };
      vi.mocked(check).mockResolvedValue(mockTauriUpdate as any);
      vi.mocked(commandGetAppSetting).mockResolvedValue(null);

      await appUpdate.initialize();
      expect(get(appUpdate).status).toBe("available");

      await appUpdate.ignoreCurrentVersion();

      expect(commandSetAppSetting).toHaveBeenCalledWith("ignored_update_version", "26.5.24");
      const state = get(appUpdate);
      expect(state.status).toBe("ignored");
      expect(state.update).toBeNull();
      expect(state.isDialogOpen).toBe(false);
    });
  });
});
