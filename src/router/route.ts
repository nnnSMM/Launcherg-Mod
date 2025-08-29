import Home from "@/views/Home.svelte";
import Memo from "@/views/Memo.svelte";
import Work from "@/views/Work.svelte";
import PlayStatusBulkEditor from "@/views/PlayStatusBulkEditor.svelte";
import Settings from "@/views/Settings.svelte";

export const routes = {
  "/": Home,
  "/works/:id": Work,
  "/memos/:id": Memo,
  "/settings/play-status": PlayStatusBulkEditor, // これが設定タブのコンテンツ
  "/settings/shortcut": Settings,
  // TODO: 404
};
