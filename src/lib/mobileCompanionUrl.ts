const configuredMobileCompanionOrigin =
  import.meta.env.VITE_MOBILE_COMPANION_ORIGIN?.trim();

export const MOBILE_COMPANION_ORIGIN =
  configuredMobileCompanionOrigin || "https://nnnsmm.github.io/Launcherg-Mod/";
export const MOBILE_COMPANION_CLIENT_VERSION = "mobile-pwa-v7";
export const SKYWAY_CONNECT_ENDPOINT = "https://launcherg.ryoha.moe/connect";

export type MobileCompanionUrlParams = {
  origin?: string;
  roomId: string;
  mode?: "controller" | "library";
  gameId?: number;
  seiyaUrl?: string;
  authToken?: string;
};

const createCompanionEntryUrl = (origin: string) => {
  const url = new URL(origin);
  const basePath = url.pathname.endsWith("/")
    ? url.pathname
    : `${url.pathname}/`;
  url.pathname = `${basePath}companion.html`;
  return url;
};

export const createMobileCompanionUrl = ({
  origin = MOBILE_COMPANION_ORIGIN,
  roomId,
  mode = "controller",
  gameId,
  seiyaUrl = "",
  authToken = "",
}: MobileCompanionUrlParams) => {
  const url = createCompanionEntryUrl(origin);
  url.searchParams.set("client", MOBILE_COMPANION_CLIENT_VERSION);
  url.searchParams.set("mode", mode);
  url.searchParams.set("roomId", roomId);

  if (gameId !== undefined) {
    url.searchParams.set("gameId", gameId.toString());
  }
  if (seiyaUrl) {
    url.searchParams.set("seiyaUrl", seiyaUrl);
  }
  if (authToken) {
    url.searchParams.set("authToken", authToken);
  }

  return url.toString();
};
