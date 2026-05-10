---
id: maintenance-routine
title: Skill Graph Maintenance Routine
type: workflow
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - decision-log
  - idea-bank
  - template-session-log
---

# Skill Graph Maintenance Routine

このグラフは完成品ではなく、作業のたびに少しずつ育てるための基盤です。

## 更新するタイミング

- 新しい機能領域を調査し、次回も使える地図ができた時。
- 重要な判断、制約、失敗パターンが見つかった時。
- ユーザーから継続的に出そうな相談テーマが増えた時。
- 検証手順やセットアップ手順が変わった時。

## 更新先

- 判断は [[decision-log]]。
- 改善候補は [[idea-bank]]。
- 作業後の学びは [[template-session-log]] の形で必要に応じて新規ノート化する。
- 既存ノートが肥大化したら、1テーマに切り出してMOCへリンクする。

## 品質基準

- frontmatterの `updated` を更新する。
- 新規ノートはMOCまたは関連ノートから辿れるようにする。
- リンク先が存在するか `npm run graph:check` で確認する。
- 実装コードと違っていたら、Markdown側を修正する。古い知識を残したい場合は理由を明記する。
