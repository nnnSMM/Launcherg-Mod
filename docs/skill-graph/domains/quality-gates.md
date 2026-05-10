---
id: quality-gates
title: Quality Gates
type: workflow
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - architecture-map
  - known-risks
---

# Quality Gates

変更の種類に応じて必要な検証を選びます。すべてを毎回実行するのではなく、壊れる範囲に合わせます。

## 常用コマンド

- Skill Graphを編集した時: `npm run graph:check`
- TypeScript/Svelteを触った時: `npm run check`
- フロントエンドロジックや既存テスト対象を触った時: `npm run test:run`
- Rust/Tauriを触った時: `cargo check --manifest-path src-tauri/Cargo.toml`
- リリース影響がある時: `npm run build`

## 手動確認が必要な領域

- トレイの右クリック、左クリック、ダブルクリック挙動。
- グローバルショートカット登録、解除、一時停止。
- スクリーンショット別ウィンドウの初期表示、再利用、閉じる挙動。
- ゲーム登録時の `.exe`、`.lnk`、ドラッグアンドドロップ、フォルダスキャン。
- 既存SQLiteデータを持つユーザーのマイグレーション。

## 完了報告の基準

実行した検証、実行しなかった検証、残リスクを分けて報告します。特にWindows実機確認が必要な変更では、コマンドが通っても完了証拠は限定的だと明示します。
