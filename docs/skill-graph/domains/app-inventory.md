---
id: app-inventory
title: App Inventory
type: domain
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - architecture-map
  - quality-gates
---

# App Inventory

作業前に場所を探すための棚卸しです。古くなったら更新します。

## Frontend

- `src/App.svelte`: ウィンドウ種別ごとのルートコンポーネント切り替え。
- `src/router/route.ts`: SPAルーティング。
- `src/views/Home.svelte`: メイン一覧。
- `src/views/Work.svelte`: ゲーム詳細。
- `src/views/Memo.svelte`: Markdownメモ編集。
- `src/views/Settings.svelte`: ショートカット設定。
- `src/views/ScreenshotWindow.svelte`: スクリーンショット専用ウィンドウ。
- `src/views/TrayMenu.svelte`: トレイメニュー。

## Frontend Modules

- `src/components/Sidebar`: 検索、インポート、ゲーム一覧。
- `src/components/Work`: 詳細、再生、画像、スクリーンショット、削除、メモ導線。
- `src/components/UI`: 共通UIと仮想スクロール。
- `src/lib`: Tauri commandラッパー、フィルタ、スクレイピング、登録処理。
- `src/store`: タブ、作品、検索、背景、メモ、状態管理。

## Backend

- `src-tauri/src/main.rs`: アプリ起動、プラグイン、トレイ、ウィンドウ、ショートカット。
- `src-tauri/src/interface/command.rs`: Tauri commandの公開面。
- `src-tauri/src/usecase`: アプリケーション処理。
- `src-tauri/src/domain`: ドメインモデルとリポジトリtrait。
- `src-tauri/src/infrastructure`: SQLite、Windows API、ファイル、プロセス実装。
- `src-tauri/src/migrations`: DB migration。

## Tests

既存のVitest対象は `src/**/*.{test,spec}.{js,ts}` です。UIコンポーネントよりも、検索、フィルタ、登録、スクレイピング、仮想スクロールなどのロジックテストが中心です。
