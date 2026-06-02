import { wrap } from "svelte-spa-router/wrap";

const homeRoute = wrap({
  asyncComponent: () => import("@/views/Home.svelte"),
});

export const routes = {
  "/": homeRoute,
  "/demo": homeRoute,
  "/stats": wrap({
    asyncComponent: () => import("@/views/Stats.svelte"),
  }),
  "/works/:id": wrap({
    asyncComponent: () => import("@/views/Work.svelte"),
  }),
  "/memos/:id": wrap({
    asyncComponent: () => import("@/views/Memo.svelte"),
  }),
  "/settings/play-status": wrap({
    asyncComponent: () => import("@/views/PlayStatusBulkEditor.svelte"),
  }),
  "/settings/shortcut": wrap({
    asyncComponent: () => import("@/views/Settings.svelte"),
  }),
  "/overlay": wrap({
    asyncComponent: () => import("@/views/Overlay.svelte"),
  }),
  // TODO: 404
};
