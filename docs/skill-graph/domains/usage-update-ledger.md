---
id: usage-update-ledger
title: USAGE Update Ledger
type: maintenance
status: active
updated: 2026-05-10
links:
  - launcherg-improvement-moc
  - maintenance-routine
---

# USAGE Update Ledger

`USAGE.md` に反映すべきユーザー向け変更を一時管理するノートです。機能追加ごとに `USAGE.md` を直接直すのではなく、まずこのノートに差分を積みます。

## 使い方

- 新機能やUI変更を入れたら、ユーザー向け説明が必要なものだけここに追加する。
- `USAGE.mdを変更して` と依頼されたら、まずこのノートを読み、対象項目を `USAGE.md` に反映する。
- スクリーンショットが必要な項目は、本文を仮決めせず、ユーザーに画像提供を依頼する。
- `USAGE.md` へ反映した項目は `status: reflected` に変えるか、履歴として完了日を追記する。
- このノートを更新したら `updated` を更新し、`npm run graph:check` を実行する。

## Pending Items

### VNDBホバープレビュー

- status: pending
- target sections: はじめに / ゲーム一覧 / 設定
- summary: ホーム画面のゲームカードにホバーすると、VNDBスクリーンショットを使ったSteam風プレビューが表示される。
- details:
  - ホバー開始後にプレビューを表示する。
  - 既存サムネイルを最初に表示し、VNDB画像はサムネイルから先に読み込んで高速表示する。
  - 画像は1秒ごとに切り替わる。
  - 画像順はVNDBの掲載順を維持する。
  - 画像比率が違う場合は切り取らず全体表示し、余白はぼかしたサムネイル背景で埋める。
  - タイトルなどの情報と黒いグラデーションは数秒後に消え、インジケーターだけ残る。
  - 最近プレイしたゲーム欄でも同じホバープレビューを使う。
- screenshot needed:
  - ホーム画面でゲームカードをホバーしてVNDBプレビューが出ている状態。
  - 画像下部のインジケーターが見える状態。
- suggested image names:
  - `images/home_hover_preview.png`

### VNDBスクリーンショット取得とキャッシュ

- status: pending
- target sections: ゲームの登録と管理 / 設定 / 注意事項
- summary: ゲーム追加時と初回ホバー時にVNDBスクリーンショット情報を取得し、SQLiteにキャッシュする。
- details:
  - ゲーム追加時にバックグラウンドでVNDBスクリーンショットを先取り取得する。
  - 失敗してもゲーム追加自体は失敗扱いにしない。
  - キャッシュが有効な場合はAPIを再呼び出ししない。
  - VNDBの候補選択はゲーム名、ゲーム名ルビ、発売日、ブランド名を使う。
  - 日本語専用スクリーンショットを優先し、ない場合は日本語を含む非MTLリリースの画像を使う。
- screenshot needed:
  - 基本的には不要。設定画面かホバープレビュー画像で補足できる。

### 表示設定

- status: pending
- target sections: 設定
- summary: タイトルバーに「表示」設定を追加し、ホーム画面プレビュー関連の設定をショートカット設定から移動した。
- details:
  - `表示設定` 画面を追加した。
  - `刺激の強いVNDBスクリーンショットも表示する` トグルを表示設定に移動した。
  - 既存設定値はそのまま引き継ぐ。
- screenshot needed:
  - 表示設定画面。
- suggested image names:
  - `images/display_settings.png`

### ホーム画面のショートカット枠と最近プレイ履歴

- status: pending
- target sections: はじめに / ゲームのプレイと記録 / 設定
- summary: ホーム画面の大きい枠は「最近プレイした一番目」ではなく、ショートカット設定済みゲームを表示する。
- details:
  - 最近プレイ履歴は先頭だけを分離せず、全件を同じ横スクロール欄に表示する。
  - 大きいヒーロー枠にはショートカット設定済みゲームを表示する。
  - ショートカット未設定時は大きい枠を表示しない。
- screenshot needed:
  - ショートカットゲームがホーム上部に表示されている状態。
  - 最近プレイ履歴が先頭を含めて横並びになっている状態。
- suggested image names:
  - `images/home_shortcut_hero.png`

### ゲームカード表示の簡素化

- status: pending
- target sections: はじめに / ゲーム一覧
- summary: 通常のゲームカード上に表示する情報をタイトルとCLEAR表示だけに整理した。
- details:
  - 通常カードでは最終プレイ日時と総プレイ時間を表示しない。
  - 詳細情報はホバープレビューや詳細画面で確認する。
- screenshot needed:
  - ホームの通常ゲームカード一覧。
  - CLEARバッジが表示されているカードがあると望ましい。
- suggested image names:
  - `images/game_cards_simplified.png`

## USAGE更新時の作業メモ

1. このノートの `Pending Items` を確認する。
2. 文章だけで反映できる項目を先に `USAGE.md` へ反映する。
3. `screenshot needed` がある項目は、必要な画像名と撮影状態をユーザーに伝えて画像提供を依頼する。
4. 画像が届いたら `USAGE.md` の該当箇所へ `<img src="./images/...">` 形式で追加する。
5. 反映済み項目の `status` を更新する。
6. `npm run graph:check` を実行する。
