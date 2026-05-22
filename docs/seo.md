# SEO Notes

updated: 2026-05-22

## 方針

Launcherg-Mod の公開ページは、検索順位を操作するためではなく、ページ内容を人間と検索エンジンの両方に正しく伝えるために整備する。対象は日本語ユーザーを主とし、広い「ゲームランチャー」一般ではなく、ノベルゲームや非Steamゲームを登録、起動、記録、整理したいユーザーの課題に寄せる。

## 現在の公開構成

- GitHub Pages 用の `npm run build-demo` が `docs/demo` に静的SPAを生成する。
- 紹介ページは demo build の `#/` と `#/landing` で表示する。
- 実アプリ体験は同じ公開URLの `#/demo` に置く。
- hash routing のため、検索意図ごとの通常パスを増やす場合は別途URL設計が必要。

## 今回の最小実装

- トップページの title / description を、VN・ノベルゲーム、プレイ時間、スクリーンショット管理が自然に伝わる内容へ調整する。
- 初期HTMLにも description と OGP / Twitter Card の最小メタを置き、JavaScript実行前でもページ概要が伝わるようにする。
- demo build で `robots.txt` と `sitemap.xml` を生成する。
- sitemap はまずトップページのみを含める。`#/demo` や `demo-data` は検索入口として扱わない。
- OGP画像は専用画像が未整備のため、壊れた参照を避けて今回は指定しない。
- Search Console のURLプレフィックス所有権確認用metaタグは `index.html` の `<head>` に直接置く。公開後は `view-source:https://nnnsmm.github.io/Launcherg-Mod/` で `google-site-verification` を検索して確認する。
- GooglebotがJavaScript実行後の本文を読めているかは、Search Console のURL検査で公開URLを検査し、レンダリング済みHTMLとスクリーンショットで確認する。

## 本番URL

既定値は GitHub Pages のプロジェクトサイト想定で `https://nnnsmm.github.io/Launcherg-Mod/` とする。公開URLを変える場合は、demo build 時に `PUBLIC_SITE_URL` または `SITE_URL` を指定して `robots.txt` と `sitemap.xml` のURLを差し替える。

## GitHub repository About

- Description: `VN・ノベルゲームのプレイ時間記録とスクリーンショット管理ができるWindows向けランチャー`
- Website: `https://nnnsmm.github.io/Launcherg-Mod/`
- Topics: `visual-novel`, `galgame`, `gaming`, `launcher`, `game-launcher`, `tauri`, `svelte`, `rust`, `typescript`, `windows`, `sqlite`, `playtime-tracker`, `screenshot-manager`

Topic は Playnite で使われている `gaming` / `launcher`、vnite で使われている `visual-novel` / `galgame` / `gaming` / `launcher` を中核にする。独自すぎる topic より、実際に使われている topic を優先する。

## 現状確認メモ

- `index.html` の `<head>` に Search Console の `google-site-verification` metaタグが入っている。
- `public/sitemap.xml` と demo build の生成結果は `https://nnnsmm.github.io/Launcherg-Mod/sitemap.xml` として公開される想定。
- トップページ本文は、ノベルゲーム/VN、非Steamゲーム、プレイ時間記録、スクリーンショット整理、Launcherg改造版の文脈を自然な範囲で含める。
- SPAの初期HTML本文は依然として薄い。title / description / OGP / sitemap で最低限の入口情報を補い、本文理解はSearch ConsoleのURL検査で確認する。

## 手動タスク

- GitHub Pages の実際の公開URLを確認する。
- Google Search Console の所有権確認を行う。
- Search Console に `sitemap.xml` を送信する。
- URL検査で `https://nnnsmm.github.io/Launcherg-Mod/` を確認し、レンダリング済みHTMLに主要本文が含まれるか確認する。
- 共有時に使う OGP 画像を用意するか、既存画像を使うか判断する。
- 実際の検索クエリを Search Console で確認し、次回改善の対象を決める。
- `/launcherg`、`/launcherg-difference`、`/migration`、機能別ページを作る場合は、hash SPAのまま増やすか通常URLで静的ページ化するかを先に決める。
