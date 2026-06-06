const isTouchDevice = () =>
  typeof navigator !== "undefined" && navigator.maxTouchPoints > 1;

export const setupMobileZoomGuard = () => {
  if (
    typeof window === "undefined" ||
    typeof document === "undefined" ||
    !isTouchDevice()
  ) {
    return;
  }

  const preventDefault = (event: Event) => {
    event.preventDefault();
  };

  document.addEventListener("gesturestart", preventDefault, { passive: false });
  document.addEventListener("gesturechange", preventDefault, {
    passive: false,
  });
  document.addEventListener("gestureend", preventDefault, { passive: false });
  document.addEventListener(
    "touchmove",
    (event) => {
      if (event.touches.length > 1) {
        event.preventDefault();
      }
    },
    { passive: false },
  );

  let lastTouchEndAt = 0;
  document.addEventListener(
    "touchend",
    (event) => {
      const now = Date.now();
      if (now - lastTouchEndAt <= 300) {
        event.preventDefault();
      }
      lastTouchEndAt = now;
    },
    { passive: false },
  );
};
