import { describe, expect, it } from "vitest";
import { createMobileCompanionUrl } from "./mobileCompanionUrl";

describe("createMobileCompanionUrl", () => {
  it("points the QR to the mobile companion controller route", () => {
    const url = createMobileCompanionUrl({
      origin: "https://example.test",
      roomId: "room-1",
    });

    expect(url).toBe(
      "https://example.test/companion.html?client=mobile-pwa-v12&mode=controller&roomId=room-1",
    );
  });

  it("keeps a subdirectory origin when building the companion entry URL", () => {
    const url = createMobileCompanionUrl({
      origin: "https://example.test/Launcherg-Mod/",
      roomId: "room-1",
    });

    expect(url).toBe(
      "https://example.test/Launcherg-Mod/companion.html?client=mobile-pwa-v12&mode=controller&roomId=room-1",
    );
  });

  it("can include an initial game without making the QR game-detail-only", () => {
    const url = createMobileCompanionUrl({
      origin: "https://example.test",
      roomId: "room-1",
      gameId: 42,
      seiyaUrl: "https://seiya.example/game?id=42",
      authToken: "token-1",
    });

    expect(url).toBe(
      "https://example.test/companion.html?client=mobile-pwa-v12&mode=controller&roomId=room-1&gameId=42&seiyaUrl=https%3A%2F%2Fseiya.example%2Fgame%3Fid%3D42&authToken=token-1",
    );
  });

  it("can explicitly create a library URL", () => {
    const url = createMobileCompanionUrl({
      origin: "https://example.test",
      roomId: "room-1",
      mode: "library",
    });

    expect(url).toBe(
      "https://example.test/companion.html?client=mobile-pwa-v12&mode=library&roomId=room-1",
    );
  });
});
