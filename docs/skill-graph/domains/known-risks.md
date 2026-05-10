---
id: known-risks
title: Known Risks
type: domain
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - architecture-map
  - quality-gates
---

# Known Risks

変更前に確認する壊れやすい領域です。

## データと互換性

- SQLite migrationは既存ユーザーのデータを直接壊しうる。
- ゲームID、キャッシュID、スクリーンショットIDの対応を崩すと、画像やメモの紐付けが破綻する。
- ファイルパスはWindows特有の表記、権限、移動、ショートカット解決を受ける。

## UIと状態

- 複数ウィンドウは `main`、`overlay`、`screenshot_window`、`tray_menu` で表示条件が異なる。
- Svelte storeの初期化順序を変えると、画面遷移や初期ロードが壊れやすい。
- 仮想スクロール、画像サイズ、背景画像はレイアウト崩れやパフォーマンス低下につながりやすい。

## Windows統合

- グローバルショートカットは登録済みキー、解除漏れ、アプリ終了時の状態に注意する。
- トレイクリックはWindowsイベント順序の影響を受ける。
- スクリーンショットやプロセス監視は環境差が出やすく、手動確認の価値が高い。

## ドキュメント

既存の日本語ドキュメントや一部ソースコメントには文字化けして見える箇所がある。無関係な文字コード修正は別タスクに分け、必要な時だけ範囲を決めて直す。
