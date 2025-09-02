import "./toast.scss";
import "./index.scss";
import "uno.css";
import TrayMenu from "@/views/TrayMenu.svelte";

const target = document.getElementById("app");
let app;
if (target) {
  app = new TrayMenu({
    target,
  });
}

export default app;
