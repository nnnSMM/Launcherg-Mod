---
id: source-skill-graphs-note
aliases:
  - source-skill-graphs-note
title: Skill Graph記事から採用した原則
type: source-note
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - maintenance-routine
---

# Skill Graph記事から採用した原則

参照元: https://note.com/l_mrk/n/ne638c6da16f3

公開部分から確認できた範囲では、Skill Graphは「複数の小さなMarkdownをWikilinkでつなぎ、AIが必要な断片だけを探索する」ための考え方です。Launcherg-Modでは、外部プラグインに依存せず、リポジトリ内のMarkdownと検証スクリプトとして実装します。

## 採用する設計

- YAML frontmatterで、ノートを開く前に用途を判断できるようにする。
- MOCでHub、Domain、Topicの見取り図を作る。
- 概要から詳細へ進む段階的開示を徹底する。
- 本文中に意味のあるWikilinkを置き、関連知識へ自然に移動できるようにする。
- 知見は大きな1ファイルへ追記し続けず、必要に応じて小さなノートへ分割する。

## 採用しないもの

- 有料部分の内容を推測して再現すること。
- Codex外部のプラグイン導入を前提にすること。
- 人間の判断を完全に置き換えること。判断の根拠は [[decision-log]] に残し、後で見直せる形にする。
