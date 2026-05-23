# UI-only vnite-inspired phase 1

This phase is UI-only. It does not add migrations or change existing stored data.

## 変更内容

- 詳細ページの `WorkLayout` と canvas 背景演出、既存 `Hero` は維持し、その下の情報面を Overview / Record / Memo / Screenshot のページに整理した。
- Hero 直下に既存の play status、ブランド、発売日、総プレイ時間、最終プレイを軽い横並びの情報行として表示し、操作バーとは上下で明示的に分けた。
- 既存の Play、管理者起動、Memo、favorite、delete、shortcut、外部リンク、screenshot 操作は同じコンポーネントと command のまま残した。
- Record ページは vnite の `RecordCard` に近い、アイコンと短いラベル中心のコンパクトな記録表示に寄せた。
- 背景画像・poster / Home 画像表示の実験は視覚崩れがあったため巻き戻し、現時点では既存の画像表示を維持している。
- Sidebar のゲーム行と検索 chip の可読性、focus 表示、icon-only button の aria label を補強した。

## 触った主なファイル

- `src/components/Work/GlassInfo.svelte`
- `src/components/Work/Info.svelte`
- `src/components/Work/Actions.svelte`
- `src/components/Work/ScreenshotGallery.svelte`
- `src/components/Sidebar/CollectionElement.svelte`

## 意図的に範囲外

- DB migration の追加、SQL schema、既存 migration の変更。
- 保存済みデータ、play time calculation、`play_status` 値の変更。
- memo / screenshot の保存方式変更。
- Rust backend command 追加、network fetching、画像取得。
- media asset table、crop 永続化、WebP / variant 生成。
- play session ledger、playtime reconciliation、extended play status。
- save management、launch preset、cloud sync、plugin system。

## 将来フェーズ候補

- extended play status は DB / enum / bulk editor / sidebar grouping をまとめて設計する。
- play session ledger と playtime reconciliation は既存 total / daily play time との互換性を先に決める。
- media asset tables、background image picker、crop / WebP / variant generation は画像 provenance と保存場所を決めてから実装する。
- save management は保存対象パス、バックアップ保持、復元 UI を別フェーズで扱う。
