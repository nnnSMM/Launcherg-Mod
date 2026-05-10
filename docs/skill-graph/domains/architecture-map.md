---
id: architecture-map
title: Architecture Map
type: domain
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - app-inventory
  - quality-gates
  - known-risks
---

# Architecture Map

このアプリはSvelteフロントエンドとTauri/Rustバックエンドで構成されています。UIは `src`、ネイティブ処理と永続化は `src-tauri` に分かれています。

## フロントエンド

- `src/App.svelte` がウィンドウラベルごとにメイン、オーバーレイ、スクリーンショット、トレイメニューを切り替える。
- `src/router/route.ts` がSPAルートを定義する。
- `src/views` は画面単位、`src/components` はUI/機能単位、`src/store` はSvelte storeの状態管理を担う。
- Tauri command呼び出しは `src/lib/command.ts` を中心に確認する。

## バックエンド

- `src-tauri/src/main.rs` がTauri Builder、プラグイン、トレイ、グローバルショートカット、ウィンドウ生成を設定する。
- `src-tauri/src/interface/command.rs` がフロントエンド公開コマンドの入口。
- `domain`、`usecase`、`infrastructure` の分割があるため、仕様変更時はもっとも近い層を読む。
- SQLiteのスキーマ変更は `src-tauri/src/migrations` に追加する。

## Windows統合

トレイ、ショートカット、プロセス監視、スクリーンショット、`.lnk` 解決、Explorer起動はWindows依存です。ここを触る時は [[known-risks]] を必ず読み、Rustの型チェックだけで完了扱いにしないで手動確認項目も出します。

## 読む順序

画面変更なら該当 `views`、直下の `components`、関連 `store`、呼び出す `lib` の順に読みます。コマンド変更なら `src/lib/command.ts`、`interface/command.rs`、対応する `usecase`、`repository`、`migration` の順に読みます。
