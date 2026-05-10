---
id: improvement-loop
title: Improvement Loop
type: workflow
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - product-context
  - architecture-map
  - quality-gates
  - decision-log
  - maintenance-routine
---

# Improvement Loop

継続改善の標準ループです。単発の修正でも、この順序を軽量に通します。

## 1. Frame

ユーザー要望を、誰のどの作業がどう良くなるかに言い換えます。曖昧な時は、最小の成功条件を1つ置きます。必要なら [[product-context]] を確認します。

## 2. Inspect

編集前に、対象ファイル、呼び出し元、共有ユーティリティ、既存テストを読む。画面変更はUIの周辺コンポーネント、データ変更はRust commandから永続化まで確認します。

## 3. Change

もっとも狭い範囲で変更します。既存パターンが複数ある場合は、より近い、より新しい、またはテストされているものを優先します。

## 4. Verify

[[quality-gates]] から、変更の種類に合う検証だけを選びます。実行できなかった検証は完了報告で明示します。

## 5. Learn

重要な判断、見つけたリスク、次回使える改善案を [[decision-log]] または [[idea-bank]] に残します。グラフを編集したら `npm run graph:check` を実行します。
