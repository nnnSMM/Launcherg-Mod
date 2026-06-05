const shouldRegisterPwaServiceWorker = () =>
  __PUBLIC_DEMO_BUILD__ &&
  !import.meta.env.DEV &&
  typeof navigator !== "undefined" &&
  "serviceWorker" in navigator;

export const isStandalonePwa = () => {
  if (typeof window === "undefined" || typeof navigator === "undefined") {
    return false;
  }

  const navigatorWithStandalone = navigator as Navigator & {
    standalone?: boolean;
  };
  return (
    window.matchMedia?.("(display-mode: standalone)").matches === true ||
    navigatorWithStandalone.standalone === true
  );
};

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
