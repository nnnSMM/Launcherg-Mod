---
id: launcherg-improvement-moc
aliases:
  - launcherg-improvement-moc
title: Launcherg-Mod Improvement MOC
type: moc
status: active
updated: 2026-05-10
links:
  - source-skill-graphs-note
  - product-context
  - architecture-map
  - improvement-loop
  - external-research
  - idea-pipeline
  - quality-gates
  - maintenance-routine
  - usage-update-ledger
  - decision-log
---

# Launcherg-Mod Improvement MOC

Launcherg-Modを継続的に改善するための見取り図です。作業を始める前にこのMOCを読み、目的に近いノートだけを辿ります。

## Hub

- [[source-skill-graphs-note]]: この基盤を作るきっかけになったSkill Graph記事の公開部分から抽出した設計原則。
- [[product-context]]: アプリが解決する問題、ユーザー価値、主要ワークフロー。
- [[architecture-map]]: Svelte/Tauri/Rust/SQLite/Windows統合の現在地。
- [[improvement-loop]]: 要望、実装、検証、学習更新までの作業ループ。
- [[external-research]]: 新機能相談やアイデア出しで外部情報を使う時の調査ルール。

## Domain

- [[idea-pipeline]]: アイデア出し、分類、評価、実験化の流れ。
- [[quality-gates]]: 変更ごとの検証基準と実行コマンド。
- [[maintenance-routine]]: この知識グラフ自体を育てる運用。
- [[decision-log]]: 重要な判断と、その理由を残す場所。
- [[known-risks]]: 変更前に確認したい壊れやすい領域。
- [[app-inventory]]: 主要ファイル、画面、コマンド、テストの棚卸し。
- [[idea-bank]]: まだ実装しない改善候補の置き場。

- [[usage-update-ledger]]: USAGE.md更新待ちのユーザー向け変更と必要スクリーンショットを管理する。

## Topic Templates

- [[template-improvement-card]]: 改善候補を1枚で整理する型。
- [[template-research-brief]]: 外部調査の問い、情報源、示唆をまとめる型。
- [[template-decision-record]]: 判断記録の型。
- [[template-session-log]]: 作業セッション後の学習更新の型。

## 読み方

目的が明確な実装作業なら [[architecture-map]] と [[quality-gates]] を優先します。仕様や体験の相談なら [[product-context]] と [[idea-pipeline]] を優先します。迷う場合は [[improvement-loop]] に戻り、まず成功条件を1つに絞ります。
