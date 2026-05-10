---
id: idea-pipeline
title: Idea Pipeline
type: workflow
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - product-context
  - external-research
  - idea-bank
  - template-improvement-card
  - template-research-brief
---

# Idea Pipeline

ユーザーから「アイデアが欲しい」と言われた時、または改善候補を整理する時の流れです。

新機能の相談では、内部の思いつきだけで答えず、必要に応じて [[external-research]] を挟みます。外部調査で得た情報は、Launcherg-Modの文脈に合う小さな実験案へ変換してから提案します。

## 分類

候補を次のいずれかに分類します。

- `workflow`: 登録、起動、記録、検索、編集などの流れを短くする。
- `reliability`: データ破損、取り込み失敗、Windows統合の不安定さを減らす。
- `visibility`: 進捗、状態、エラー、次の行動を分かりやすくする。
- `data`: SQLite、キャッシュ、メタデータ、画像、スクリーンショットの扱いを良くする。
- `performance`: スキャン、検索、画像表示、仮想スクロールを軽くする。
- `maintenance`: テスト、ドキュメント、検証、自動化で将来の変更を楽にする。
- `research`: 外部情報、競合事例、プラットフォーム制約を調べてから判断する。

## 外部調査を使う場面

次の相談では [[external-research]] を使います。

- 新機能の方向性、優先順位、UIパターンを相談する時。
- Tauri、Windows API、Svelte、ライブラリなど、最新情報や仕様変更の影響がありそうな時。
- 競合・類似アプリの機能、価格、UX、運用から学びたい時。
- ユーザーが時間やお金を使う判断につながる提案をする時。
- 公式ドキュメント、リリースノート、実例へのリンクが必要な時。

## 評価

各候補は `impact`、`effort`、`confidence`、`risk` の4軸で短く評価します。外部調査を使った場合は、根拠と不確実性も添えます。実装へ進める前に、[[template-improvement-card]] の形へ落とします。

## 出力形式

アイデア出しの返答では、最初に3から5件に絞ります。それぞれについて「狙い」「根拠」「最初の小さな実装」「検証方法」を出します。大きすぎる案は、最初の1週間以内に終わる単位へ分割します。

## 保留

今は着手しないが価値がありそうな案は [[idea-bank]] に置きます。保留理由も書き、同じ議論を繰り返さないようにします。
