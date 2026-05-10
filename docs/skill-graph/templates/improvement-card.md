---
id: template-improvement-card
aliases:
  - template-improvement-card
title: Improvement Card Template
type: template
status: active
updated: 2026-05-10
links:
  - idea-pipeline
  - external-research
  - quality-gates
---

# Improvement Card Template

改善候補を実装前に1枚へ絞るためのテンプレートです。

```md
---
id: improvement-YYYYMMDD-short-name
title: 改善案タイトル
type: improvement
status: proposed
updated: YYYY-MM-DD
links:
  - product-context
  - quality-gates
---

# 改善案タイトル

## Problem

誰のどの作業が、今どう困っているか。

## Smallest Useful Change

最初に入れる最小変更。

## Scope

触る予定のファイル、触らないもの。

## Evaluation

- Impact:
- Effort:
- Confidence:
- Risk:

## Evidence

外部調査を使った場合は、参照元、確認日、Launcherg-Mod向けの解釈を書く。使っていない場合は、その理由を書く。

## Verification

実行するコマンドと手動確認。

## Follow-up

この変更後に見える次の改善候補。
```
