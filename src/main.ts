import "virtual:uno.css";
import "@unocss/reset/tailwind-compat.css";
import "./index.scss";
import "tippy.js/dist/tippy.css";
import "simplebar/dist/simplebar.css";
import "./toast.scss";
import App from "./App.svelte";
import { registerPwaServiceWorker } from "@/lib/pwa";
import { setupMobileZoomGuard } from "@/lib/mobileZoomGuard";

registerPwaServiceWorker();
setupMobileZoomGuard();

const app = new App({
  // @ts-expect-error
  target: document.getElementById("app"),
});

export default app;
