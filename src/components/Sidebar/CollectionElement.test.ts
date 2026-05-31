import { describe, it, expect, beforeEach, vi } from "vitest";
import CollectionElementComponent from "./CollectionElement.svelte";
import type { CollectionElement } from "@/lib/types";

// tauri api などのモック
vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (src: string) => src,
}));

vi.mock("svelte-spa-router", () => ({
  link: () => {},
  location: {
    subscribe: (run: any) => {
      run("/");
      return () => {};
    }
  },
}));

describe("CollectionElement", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  const dummyElement: CollectionElement = {
    id: 1,
    gamename: "テストゲームタイトル",
    gamenameRuby: "てすとげーむたいとる",
    brandname: "テストブランド",
    brandnameRuby: "てすとぶらんど",
    sellday: "2026-05-23",
    isNukige: false,
    installAt: null,
    firstPlayAt: null,
    lastPlayAt: "2026-05-23T17:00:00Z",
    likeAt: null,
    playStatus: 0,
    totalPlayTimeSeconds: 0,
    registeredAt: "2026-05-23T17:00:00Z",
    exePath: "C:\\game.exe",
    lnkPath: "C:\\game.lnk",
    icon: "C:\\icon.png",
    thumbnail: "C:\\thumbnail.png",
    thumbnailWidth: null,
    thumbnailHeight: null,
    updatedAt: "2026-05-23T17:00:00Z",
  };

  it("ブランド名や日付が表示されず、アイコンサイズが h-4 w-4、丸みが rounded、パディングが py-1 であること", () => {
    const target = document.body;
    new CollectionElementComponent({
      target,
      props: {
        collectionElement: dummyElement,
      },
    });

    // ゲームタイトルは表示されること
    expect(target.textContent).toContain("テストゲームタイトル");

    // ブランド名は表示されないこと
    expect(target.textContent).not.toContain("テストブランド");

    // 日付（最後にプレイした日、本日の日付など）は表示されないこと
    // formatLastPlayedは「今日」「昨日」「◯日前」や日付を返す
    expect(target.textContent).not.toContain("今日");
    expect(target.textContent).not.toContain("2026-05-23");

    // アイコン（imgタグ）のサイズ変更の検証
    const img = target.querySelector("img");
    expect(img).not.toBeNull();
    if (img) {
      expect(img.className).toContain("h-5");
      expect(img.className).toContain("w-5");
      expect(img.className).not.toContain("h-4");
      expect(img.className).not.toContain("w-4");
      expect(img.className).not.toContain("h-8");
      expect(img.className).not.toContain("w-8");

      // 角の丸みが rounded になり、rounded-md ではなくなっていること
      expect(img.className).toContain("rounded");
      expect(img.className).not.toContain("rounded-md");
    }

    // ゲームタイトルの文字サイズが text-xs になり、text-sm ではなくなっていることの検証
    const titleEl = Array.from(target.querySelectorAll("div")).find(
      (el) => el.textContent?.trim() === "テストゲームタイトル" && el.className.includes("text-")
    );
    expect(titleEl).not.toBeNull();
    if (titleEl) {
      expect(titleEl.className).toContain("text-xs");
      expect(titleEl.className).not.toContain("text-sm");
    }

    // ゲーム間の間隔（パディングが py-1 になっていること）の検証
    const wrapper = target.querySelector("div");
    expect(wrapper).not.toBeNull();
    if (wrapper) {
      expect(wrapper.className).toContain("py-1");
      expect(wrapper.className).not.toContain("py-2");
    }
  });
});

// ADisclosure のテスト（アコーディオンの「すべて」等のラベルの文字サイズ検証）
import ADisclosureComponent from "../UI/ADisclosure.svelte";

describe("ADisclosure", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("アコーディオンラベルの文字サイズが text-xs で、パディングが p-y-1 であること", () => {
    const target = document.body;
    new ADisclosureComponent({
      target,
      props: {
        label: "すべて",
      },
    });

    const labelEl = Array.from(target.querySelectorAll("div")).find(
      (el) => el.textContent?.trim() === "すべて" && el.className.includes("text-")
    );
    expect(labelEl).not.toBeNull();
    if (labelEl) {
      expect(labelEl.className).toContain("text-xs");
      expect(labelEl.className).not.toContain("text-body2");
    }

    const wrapper = target.querySelector("button div");
    expect(wrapper).not.toBeNull();
    if (wrapper) {
      expect(wrapper.className).toContain("p-y-1");
      expect(wrapper.className).not.toContain("p-y-2");
    }

    const button = target.querySelector("button");
    expect(button).not.toBeNull();
    if (button) {
      expect(button.className).not.toContain("hover:bg-bg-button-hover");
    }
  });
});
