const CACHE_NAME = "launcherg-pwa-shell-mobile-pwa-v4";
const PRECACHE_ASSETS = Array.isArray(self.__LAUNCHERG_PWA_ASSETS__)
  ? self.__LAUNCHERG_PWA_ASSETS__
  : [];
const APP_SHELL = [
  "./",
  "./index.html",
  "./companion.html",
  "./icon.png",
  "./manifest.webmanifest",
  ...PRECACHE_ASSETS,
];
const CACHEABLE_DESTINATIONS = new Set(["script", "style", "worker", "font"]);

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) => cache.addAll(APP_SHELL))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key !== CACHE_NAME)
            .map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

const cacheShellResponse = async (request, response) => {
  if (!response || !response.ok) {
    return response;
  }

  const cache = await caches.open(CACHE_NAME);
  await cache.put(request, response.clone());
  return response;
};

const networkFirstDocument = async (request) => {
  try {
    return await cacheShellResponse(request, await fetch(request));
  } catch {
    const url = new URL(request.url);
    if (url.pathname.endsWith("/companion.html")) {
      return (
        (await caches.match("./companion.html")) ||
        (await caches.match(request, { ignoreSearch: true })) ||
        (await caches.match("./index.html")) ||
        (await caches.match("./"))
      );
    }

    return (
      (await caches.match(request)) ||
      (await caches.match("./index.html")) ||
      (await caches.match("./"))
    );
  }
};

const cacheFirstAsset = async (request) => {
  const cached = await caches.match(request);
  if (cached) {
    return cached;
  }

  return cacheShellResponse(request, await fetch(request));
};

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") {
    return;
  }

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) {
    return;
  }

  if (request.mode === "navigate") {
    event.respondWith(networkFirstDocument(request));
    return;
  }

  if (CACHEABLE_DESTINATIONS.has(request.destination)) {
    event.respondWith(cacheFirstAsset(request));
  }
});
