import TrayMenu from "./components/TrayMenu.svelte";

const app = new TrayMenu({
  target: document.getElementById("app"),
});

export default app;
