const shouldRegisterPwaServiceWorker = () =>
  __PUBLIC_DEMO_BUILD__ &&
  !import.meta.env.DEV &&
  typeof navigator !== "undefined" &&
  "serviceWorker" in navigator;

export const registerPwaServiceWorker = () => {
  if (!shouldRegisterPwaServiceWorker()) {
    return;
  }

  void navigator.serviceWorker
    .register("./sw.js", { scope: "./" })
    .catch((error) => {
      console.warn("[pwa] Failed to register service worker.", error);
    });
};
