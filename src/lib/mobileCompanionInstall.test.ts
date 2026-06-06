import { describe, expect, it } from "vitest";
import {
  configureMobileCompanionInstallManifest,
  createMobileCompanionInstallManifest,
  createMobileCompanionInstallStartUrl,
} from "./mobileCompanionInstall";

describe("createMobileCompanionInstallStartUrl", () => {
  it("keeps the room in the installed launch URL", () => {
    const query = new URLSearchParams({
      client: "mobile-pwa-v13",
      mode: "library",
      roomId: "room-1",
      gameId: "42",
      authToken: "short-lived-token",
    });

    expect(
      createMobileCompanionInstallStartUrl(
        "https://example.test/Launcherg-Mod/#/companion?roomId=room-1",
        query,
      ),
    ).toBe(
      "https://example.test/Launcherg-Mod/#/companion?client=mobile-pwa-v13&mode=library&roomId=room-1&gameId=42&authToken=short-lived-token",
    );
  });

  it("keeps the QR SkyWay token for installed launch reconnects", () => {
    const query = new URLSearchParams({
      roomId: "room-1",
      mode: "controller",
      authToken: "short-lived-token",
    });
    const startUrl = createMobileCompanionInstallStartUrl(
      "https://example.test/Launcherg-Mod/#/companion",
      query,
    );

    expect(startUrl).toContain("roomId=room-1");
    expect(startUrl).toContain("mode=controller");
    expect(startUrl).toContain("authToken=short-lived-token");
  });

  it("does not create an install URL without a room", () => {
    expect(
      createMobileCompanionInstallStartUrl(
        "https://example.test/Launcherg-Mod/#/companion",
        new URLSearchParams(),
      ),
    ).toBeNull();
  });
});

describe("createMobileCompanionInstallManifest", () => {
  it("uses absolute same-origin URLs for the dynamic manifest", () => {
    const manifest = createMobileCompanionInstallManifest(
      "https://example.test/Launcherg-Mod/#/companion?roomId=room-1",
      "https://example.test/Launcherg-Mod/#/companion?roomId=room-1",
    );

    expect(manifest.id).toBe("https://example.test/Launcherg-Mod/companion.html");
    expect(manifest.start_url).toBe(
      "https://example.test/Launcherg-Mod/#/companion?roomId=room-1",
    );
    expect(manifest.scope).toBe("https://example.test/Launcherg-Mod/");
    expect(manifest.icons[0].src).toBe(
      "https://example.test/Launcherg-Mod/icon.png",
    );
  });
});

describe("configureMobileCompanionInstallManifest", () => {
  it("replaces the page manifest with a room-specific manifest", () => {
    document.head.innerHTML = '<link rel="manifest" href="./manifest.webmanifest">';
    const query = new URLSearchParams({
      roomId: "room-1",
      mode: "controller",
      authToken: "short-lived-token",
    });

    const startUrl = configureMobileCompanionInstallManifest(query, document, {
      href: "https://example.test/Launcherg-Mod/#/companion?roomId=room-1",
    } as Location);
    const link = document.querySelector<HTMLLinkElement>('link[rel="manifest"]');

    expect(startUrl).toBe(
      "https://example.test/Launcherg-Mod/#/companion?client=mobile-pwa-v13&mode=controller&roomId=room-1&authToken=short-lived-token",
    );
    expect(link?.href).toContain("data:application/manifest+json");
    expect(link?.dataset.launchergDynamicCompanionManifest).toBe("true");
    const manifest = JSON.parse(
      decodeURIComponent(link?.href.split(",")[1] ?? ""),
    );
    expect(manifest.start_url).toBe(
      "https://example.test/Launcherg-Mod/#/companion?client=mobile-pwa-v13&mode=controller&roomId=room-1&authToken=short-lived-token",
    );
  });
});
