---
id: decision-log
title: Decision Log
type: log
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - template-decision-record
  - source-skill-graphs-note
---

# Decision Log

Launcherg-Modの継続改善に影響する判断を残す場所です。量が増えたら、判断ごとに別ノートへ分割します。

## 2026-05-10: リポジトリ内MarkdownとしてSkill Graphを構築する

- Context: ユーザーはSkill Graph記事を参考に、アプリを継続的に改善する基盤を求めている。
- Decision: 外部プラグイン導入ではなく、`.codex/skill-graph` にMarkdownグラフを置き、`npm run graph:check` で最低限の構造を検証する。
- Rationale: このリポジトリと同じ場所に置けば、Codex作業時に毎回参照しやすく、バージョン管理もしやすい。外部ツールがなくても機能する。
- Consequence: 自動生成や高度な探索は持たないため、人間とエージェントが更新ルールを守る必要がある。
- Links: [[source-skill-graphs-note]], [[maintenance-routine]]

## 2026-05-10: 改善基盤は日本語で運用する

- Context: リポジトリのエージェント指示では、思考は英語、応答は日本語とされている。
- Decision: Skill Graphの本文、テンプレート、運用説明は日本語にする。
- Rationale: ユーザーとの継続的な相談、アイデア出し、判断記録をそのまま蓄積しやすい。
- Consequence: ファイル名とIDはリンク安定性のためASCII寄りにする。
- Links: [[template-decision-record]]
