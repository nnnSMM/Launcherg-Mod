import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

type WebAppManifest = {
  id?: string;
  start_url?: string;
};

describe("mobile companion web app manifest", () => {
  it("lets installed PWA launches keep the QR URL that was added", async () => {
    const manifestPath = resolve("public/manifest.webmanifest");
    const manifest = JSON.parse(
      await readFile(manifestPath, "utf-8"),
    ) as WebAppManifest;

    expect(manifest.id).toBe("./companion.html");
    expect(manifest.start_url).toBeUndefined();
  });
});
