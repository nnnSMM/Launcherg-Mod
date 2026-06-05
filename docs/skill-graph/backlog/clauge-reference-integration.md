---
id: clauge-reference-integration
title: Clauge Reference Integration Plan
type: backlog
status: active
updated: 2026-06-05
links:
  - idea-bank
  - architecture-map
  - known-risks
  - quality-gates
---

# Clauge Reference Integration Plan

Clauge はコードを取り込む対象ではなく、Launcherg-Mod の開発体験と運用性を良くするための設計参考として扱う。ライセンスが PolyForm Noncommercial なので、実装は既存コードと要件から作り直す。

## 現状に対する見立て

すでに土台があるもの:

- 更新通知: `src/store/update.ts` と `src/components/Update/*` に検知、無視、進捗、リリースリンクがある。
- 保存値の復旧: `src/lib/utils.ts` に localStorage の JSON 復旧、期限付きキャッシュ、store 化がある。
- ログ: `src-tauri/src/main.rs` で `tauri-plugin-log` を有効化している。
- ショートカット設定: `src/views/Settings.svelte` と Rust 側の登録処理がある。

不足しているもの:

- フロントエンド例外、Tauri command 失敗、Rust 側エラーの見せ方が分散している。
- `showErrorToast(`${error}`)` 型の生エラー表示が多く、ユーザーが次に何をすればよいか分かりにくい。
- ショートカット入力は保存と登録が密結合で、無効なキーの扱いが分かりにくい。
- 更新後に「何が変わったか」をアプリ内で確認する導線が薄い。
- localStorage、SQLite app settings、Svelte store のキーが分散しており、将来の移行時に漏れやすい。

## 組み込み順

1. 診断と失敗表示を先に整える。
   `src/lib/errors.ts` のような薄い分類レイヤーを作り、手動追加、自動インポート、ショートカット保存、スクリーンショット操作、更新処理の catch から順に使う。分類は決定的な文字列、例外型、失敗箇所で行い、AI には任せない。ここは [[known-risks]] の Windows 統合リスクにも効く。

2. ログの取り出し方を確認し、足りなければ最小の橋を作る。
   既存の `tauri-plugin-log` が WebView 例外をどこまで残すかを確認する。足りなければ `app_log` command または Tauri log API 経由で `window.error` と `unhandledrejection` を送る。ログ表示 UI は後回しにし、まずサポート用の保存先とマスク方針を決める。

3. 既存 UI に薄く足せるものを入れる。
   `src/store/update.ts` のメタデータを使って What's New モーダルを追加する。ショートカット一覧は `src/views/Settings.svelte` の保存値を読んでヘルプ表示するだけに留め、設定画面の再設計はしない。

4. 保存キーの棚卸しを行う。
   いきなり共通設定基盤へ移行せず、localStorage key、SQLite app setting key、永続化 store を一覧化する。次に新規キーだけを定数化し、既存キーの移行は壊れた値の復旧パスが確認できてから行う。

5. データ持ち出しは export-only から始める。
   登録ゲーム、プレイ状態、メモ、スクリーンショット参照、主要設定を JSON で出す。import はパス不一致、画像ファイル、DB migration と衝突しやすいので、形式が固まるまで延期する。

6. Codex 作業運用は Markdown のまま軽く始める。
   ボード UI や SQLite 管理は作らず、重要な調査、判断、未検証事項を [[idea-bank]] と decision log に残す。MCP 化は、Codex が実際に何度も同じ情報を探している証拠が出てから検討する。

## 当面やらないもの

- Clauge 風の Workspace/Board UI をアプリ本体に入れる。
- ローカル MCP サーバーを最初から常駐させる。
- in-app AI、クラウド同期、エージェント管理を入れる。
- 既存の設定画面や更新処理を大きく作り直す。

## 最初の実装候補

最初は「エラー分類と診断ログの最小整備」を 1 タスクにする。成功条件は、代表的な失敗時にユーザー向けメッセージ、開発者向けログ、再現に必要な操作箇所が分かれること。検証は `npm run check`、関連 unit test、Tauri command の手動確認を [[quality-gates]] に沿って行う。
