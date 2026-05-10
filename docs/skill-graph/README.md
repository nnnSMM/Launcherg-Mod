---
id: skill-graph-readme
aliases:
  - skill-graph-readme
title: Launcherg-Mod Skill Graph README
type: guide
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
---

# Launcherg-Mod Skill Graph

このフォルダは、Launcherg-Modを継続的に改善するためのローカルな知識グラフです。

入口は [[launcherg-improvement-moc]] です。まずMOCで必要そうな領域を選び、各ノートのfrontmatterと短い概要だけを見てから、必要な本文へ進みます。

## 運用ルール

- 1ファイル1テーマを基本にする。
- すべてのMarkdownにYAML frontmatterを置く。
- 関連ノートは本文中の自然な文脈で `[[note-id]]` としてリンクする。
- 大きな方針変更、判断、失敗から得た知見は [[decision-log]] に追記する。
- アイデア出しや優先順位付けは [[idea-pipeline]] と [[idea-bank]] を使う。
- このグラフを編集したら `npm run graph:check` でリンク切れとfrontmatterを検査する。

## Obsidianで使う

1. Obsidianで `Open folder as vault` を選び、`E:\Launcherg-Mod\docs\skill-graph` を開く。
2. 最初に `MOC.md` を開く。表示名では [[launcherg-improvement-moc]] が入口。
3. 左サイドバーの検索で `type: workflow`、`status: active`、`idea` などを探す。
4. グラフビューを開き、MOCから各ノートへのつながりを見る。
5. ノートを編集したら、リポジトリルートで `npm.cmd run graph:check` を実行する。

Obsidianはvault内に `.obsidian` フォルダを作ります。設定をGitに入れたくない場合は、ローカルの `.git/info/exclude` に `docs/skill-graph/.obsidian/` を追加します。

## 設計メモ

この構成は、公開されているSkill Graph解説で説明されていたYAML frontmatter、MOC、段階的開示、意味付けされたWikilinkの考え方を、Codexで扱いやすいリポジトリ内Markdownに落とし込んだものです。
