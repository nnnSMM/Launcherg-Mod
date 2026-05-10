---
id: idea-bank
title: Idea Bank
type: backlog
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - idea-pipeline
  - template-improvement-card
  - known-risks
---

# Idea Bank

すぐに実装しない改善候補を置く場所です。実装候補に上げる時は [[template-improvement-card]] の形に整理します。

## 候補

### 文字化けしている日本語ドキュメントの復旧計画

- Type: `maintenance`
- 狙い: README、USAGE、コメントの可読性を戻し、ユーザー向け説明と将来の開発効率を上げる。
- 注意: 元のエンコーディングと意図を確認してから範囲を決める。機能変更と混ぜない。

### Windows統合の手動QAチェックリスト

- Type: `reliability`
- 狙い: トレイ、ショートカット、スクリーンショット、プロセス監視の回帰確認を定型化する。
- 最初の一歩: [[quality-gates]] の手動確認項目を `docs` またはSkill Graphの専用ノートへ具体化する。

### インポート処理の失敗理由を見える化する

- Type: `visibility`
- 狙い: フォルダスキャンや手動追加で、何が追加されなかったのかをユーザーが理解できるようにする。
- 最初の一歩: `create_elements_in_pc` からフロントへ返す情報と既存UIの進捗表示を調査する。

### スクリーンショット管理の操作回帰テストを増やす

- Type: `reliability`
- 狙い: 別ウィンドウの初期引数、並び替え、削除、インポートの破損を減らす。
- 注意: Tauri実機確認が必要な範囲と、純粋なロジックテストで守れる範囲を分ける。

### ゲーム詳細画面の情報密度を見直す

- Type: `workflow`
- 狙い: 起動、メモ、スクリーンショット、パス修正、削除などの操作を迷わず行えるようにする。
- 最初の一歩: `src/components/Work` の構造と現行スクリーンショットを確認し、変更なしのUXレビューを行う。

### DB migrationの互換性チェックを強化する

- Type: `data`
- 狙い: 既存DBに対するmigration追加時の事故を減らす。
- 最初の一歩: 現在の `src-tauri/src/migrations` と `launcherg_sqlite.db3` の扱いを調査する。
