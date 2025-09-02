import "./toast.scss";
import Tray from "./Tray.svelte";

const app = new Tray({
  target: document.getElementById("app"),
});

export default app;
