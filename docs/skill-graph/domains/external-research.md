---
id: external-research
title: External Research Workflow
type: workflow
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - idea-pipeline
  - product-context
  - template-research-brief
  - idea-bank
---

# External Research Workflow

新機能相談やアイデア出しで、外部情報を使うためのワークフローです。目的は、外部の流行や競合をそのまま真似ることではなく、Launcherg-Modのユーザー価値に合う仮説へ変換することです。

## 調査する前に決めること

- 調査問い: 何を知れば判断できるか。
- 対象ユーザー: Launcherg-Modのどの利用シーンに関係するか。
- 意思決定: 調査結果で「やる」「やらない」「小さく試す」のどれを決めたいか。
- 制約: Windowsローカルアプリ、Tauri、既存SQLite、既存ユーザーデータ、個人情報を前提にする。

## 優先する情報源

1. 公式ドキュメント、API仕様、リリースノート。
2. 類似アプリや競合の公式サイト、ヘルプ、公開ロードマップ、リリースノート。
3. GitHub issue、Discussion、READMEなど、実装や利用者の課題が見える一次情報。
4. 信頼できる技術記事やレビュー。使う場合は、個人の意見と事実を分ける。

技術仕様、価格、対応OS、API、最新バージョン、規約、ストア要件など変わりやすい情報は、必ず外部確認してから提案します。

## 調査の進め方

1. まず [[product-context]] と [[idea-pipeline]] で、Launcherg-Mod側の文脈を確認する。
2. 調査問いを1から3個に絞る。
3. 外部情報を確認し、出典URL、確認日、要点、不確実性をメモする。
4. 得た情報を「このアプリで小さく試すなら何か」に変換する。
5. 実装候補にするなら [[template-improvement-card]]、調査メモとして残すなら [[template-research-brief]] を使う。

## 出力基準

外部調査込みのアイデア提案では、次を必ず含めます。

- 参照した情報源へのリンク。
- そこから分かった事実。
- Launcherg-Mod向けの解釈。
- 最初に作るならどこまでに絞るか。
- 検証方法と、まだ確かでない点。

## 注意

- 競合機能を見つけても、そのまま大きく実装しない。
- 最新情報が必要な話では、記憶だけで断定しない。
- ユーザーのローカルデータ、プレイ履歴、画像、メモに関わる案では、プライバシーとバックアップを先に確認する。
- 規約や著作権に関わる取得、スクレイピング、画像利用は、根拠を確認してから進める。
