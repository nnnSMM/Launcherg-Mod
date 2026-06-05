import {
  MOBILE_COMPANION_CLIENT_VERSION,
  type MobileCompanionUrlParams,
} from "@/lib/mobileCompanionUrl";

const DYNAMIC_MANIFEST_MARKER = "launchergDynamicCompanionManifest";

const optionalStartUrlKeys: Array<keyof MobileCompanionUrlParams | "mode"> = [
  "gameId",
  "seiyaUrl",
];

export const createMobileCompanionInstallStartUrl = (
  href: string,
  query: URLSearchParams,
) => {
  const roomId = query.get("roomId")?.trim();
  if (!roomId) {
    return null;
  }

  const url = new URL("./", href);
  const params = new URLSearchParams();
  params.set(
    "client",
    query.get("client")?.trim() || MOBILE_COMPANION_CLIENT_VERSION,
  );
  params.set("mode", query.get("mode")?.trim() || "library");
  params.set("roomId", roomId);

  optionalStartUrlKeys.forEach((key) => {
    const value = query.get(key)?.trim();
    if (value) {
      params.set(key, value);
    }
  });

  url.search = "";
  url.hash = `/companion?${params.toString()}`;
  return url.toString();
};

export const createMobileCompanionInstallManifest = (
  href: string,
  startUrl: string,
) => {
  const baseUrl = new URL("./", href);
  const companionIdUrl = new URL("./companion.html", baseUrl);
  const iconUrl = new URL("./icon.png", baseUrl);

  return {
    name: "Launcherg Mobile Companion",
    short_name: "Launcherg",
    description: "Launcherg-Mod companion PWA for mobile play support.",
    id: companionIdUrl.toString(),
    start_url: startUrl,
    scope: baseUrl.toString(),
    display: "standalone",
    orientation: "any",
    background_color: "#111827",
    theme_color: "#111827",
    categories: ["games", "productivity", "utilities"],
    icons: [
      {
        src: iconUrl.toString(),
        sizes: "1660x1660",
        type: "image/png",
        purpose: "any",
      },
      {
        src: iconUrl.toString(),
        sizes: "1660x1660",
        type: "image/png",
        purpose: "maskable",
      },
    ],
  };
};

export const configureMobileCompanionInstallManifest = (
  query: URLSearchParams,
  doc: Document = document,
  location: Location = window.location,
) => {
  const startUrl = createMobileCompanionInstallStartUrl(
    location.href,
    query,
  );
  if (!startUrl) {
    return null;
  }

  const manifest = createMobileCompanionInstallManifest(
    location.href,
    startUrl,
  );
  const encodedManifest = encodeURIComponent(JSON.stringify(manifest));
  let link = doc.querySelector<HTMLLinkElement>('link[rel="manifest"]');

  if (!link) {
    link = doc.createElement("link");
    link.rel = "manifest";
    doc.head.appendChild(link);
  }

  link.href = `data:application/manifest+json;charset=utf-8,${encodedManifest}`;
  link.dataset[DYNAMIC_MANIFEST_MARKER] = "true";
  return startUrl;
};
