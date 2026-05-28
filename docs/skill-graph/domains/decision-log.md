---
id: decision-log
title: Decision Log
type: log
status: active
updated: 2026-05-27
links:
  - launcherg-improvement-moc
  - template-decision-record
  - source-skill-graphs-note
---

# Decision Log

## 2026-05-28: ゲーム詳細画面のメモ欄プレビュー化と別タブ編集のプレビュー強化

- Context: ゲーム詳細画面のメモ欄はこれまでEasyMDEエディタが直接埋め込まれており、起動に負荷がかかるほか、閲覧するだけの場面でもエディタが表示されてしまっていた。一方で別タブ（Memo.svelte）は編集用画面でありながらプレビューツールバーがなく、編集しながら記述内容をプレビューする手段が不足していた。
- Decision: ゲーム詳細画面（Info.svelte）のメモ欄ではEasyMDEによるテキストエリアを完全に削除し、安全かつ美麗なマークダウンパース（marked）によるHTMLでの「内容」表示へと変更する。表示するメモは、ゲーム切り替え時に localStorage から $memo ストアへ自動初期同期するリアクティブステートメントを導入し、ストア側の変更と完全連動するようにした。別タブ of 編集画面（Memo.svelte）は編集専用として維持しつつ、EasyMDEのツールバーに preview、side-by-side を追加し、いつでもプレビューができるように改善する（不要な fullscreen は除外）。また、標準の flexbox と EasyMDE が競合して左右分割表示（side-by-side）が崩れる問題に対し、モダンCSSの :has(.CodeMirror-sided) 擬似クラスを用いて親コンテナをCSSグリッド（2カラム配置）へと動的に切り替えることで、ズレのない100%確実な横並び配置を実現した。さらに、表示領域およびプレビュー領域を最高品質の美しさに仕上げるため、見出しのグラデーションテキスト、極細の枠線付きコードブロック、丸みのある陰影画像、半透明のプレミアムガラスモーフィズムパネル（backdrop-blur-lg、inset-shadow-sm、border-white/10）などを SCSS と HTML に適用してビジュアル面を極限まで美化した。マークダウン変換処理は新規モジュール `src/lib/markdown.ts` に切り出し、Tauri環境のローカル画像パスを表示するため convertFileSrc を適用するカスタムレンダラーを実装し、t_wada氏のTDDスタイルに従ってテスト駆動で開発・保証した。
- Rationale: 閲覧が中心の詳細画面では表示を軽量化・プレビューのみにして誤操作を防ぎ、本格的な編集は「メモを開く」から別タブエディタで行うように役割を分離することで、操作性とパフォーマンスが向上する。また、マークダウン変換モジュールをテスト駆動で開発することで、Tauri環境固有の画像パス変換処理などのデグレを防ぎ、継続的な保守性を高められる。
- Consequence: 詳細画面で直接メモを書き換えることはできなくなるため、編集する場合は必ず「メモを開く」ボタンから別タブ編集画面へ遷移する必要がある。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-27: demo用 worksData は build-demo 前に本体スクレイパーで全再生成する

- Context: 公開 demo の作品詳細は `src/mock/worksData.json` に固定されているが、説明文や統計値は外部サイト側と本体パーサー改善に追従させたい。手書きJSONや簡易パーサーを別に持つと、本体の `getWorkByScrape` と乖離しやすい。
- Decision: `npm run build-demo` の前段で Vitest スクリプトを実行し、デモ対象IDごとに `getWorkByScrape(id)` を呼んで `worksData.json` を丸ごと再構築する。Vitest内では `@tauri-apps/plugin-http` を Node `fetch` に差し替え、FANZA/DLsite向けCookieを付与する。ErogeScape `gamelist` の外部ID欠落に左右されないよう、`scrapeSql` の `gamelist` 取得だけを手動マッピングで返す。公開demoの詳細画面では `@/store/works` をデモ用ストアへ確実に差し替え、再生成済み `worksData.json` を直接使う。demo成果物のJS/CSSファイル名には `DEMO_BUILD_ID`、`GITHUB_SHA`、または時刻由来IDを含める。
- Rationale: 本体と同じHTML解析・説明文抽出・統計/スタッフ取得を使うため、demoだけ別実装になるリスクを避けられる。手動マッピングはdemo収録作品に限定し、外部サイトの登録漏れ対策として閉じ込める。
- Consequence: demoビルドは外部サイトの応答に依存し、取得失敗時は古いJSONを使い回さずビルドを止める。公開側は毎回異なるasset URLになり、ブラウザやPages CDNが古いJSを掴み続ける問題を避ける。デモ対象を追加する時は、Vitest側の外部IDマッピングと `src/mock/tauri-http.ts` のデモHTTPマッピングを同時に更新する。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-26: 日本語説明文は販売ID直行に限定し公式サイトfallbackを廃止する

- Context: `gamelist` の `dmm` / `dlsite_id` / `dlsite_domain` / `steam` で販売ページを確定できる一方、`shoukai` 公式サイトfallbackはページ構造差が大きく、ストーリー以外を拾う精度問題があった。FANZA本文は複数の `.text-overflow` や装飾区切りを含む場合があり、先頭要素だけの抽出では欠落リスクがあった。また Tauri の `plugin-http` は標準設定だと `Cookie` / `Referer` を送らず、FANZAの年齢確認を突破できない。
- Decision: 日本語説明文取得は FANZA -> DLsite -> Steam の販売ID直行のみとし、公式サイトfallbackを取得経路から外す。FANZAは該当する `.text-overflow` を全て結合し、本文中の装飾区切りだけで収集を終了しない。FANZA取得のため `tauri-plugin-http` の `unsafe-headers` feature を有効化し、年齢確認Cookieを明示送信する。`steam` がDB未登録の作品だけ、`shoukai` 公式サイト本文は保存せずSteamストアリンク抽出のみに使う。DLsiteは確定済み `dlsite_id` の通常 `work` ページが消えている場合だけ、同じIDの `announce` ページを試す。作品情報キャッシュは抽出仕様バージョン付きで7日保持し、バージョン不一致または期限切れなら再取得する。
- Rationale: 対象確定はDB由来IDの方が再現性が高く、公式サイト探索は誤取得の影響が大きい。販売ページ内のストーリー抽出は、複数段落と区切り線を許容する方が「取得済みなのに途中で欠ける」リスクを下げられる。FANZAはCookieが落ちると年齢確認HTMLになり、説明文抽出が全滅する。
- Consequence: 公式サイトしか情報がない作品では説明文は空になる。今後の追加取得先も、名前検索や公式探索ではなく、DBで確定できるID/URLがある場合に限定する。
- Links: [[architecture-map]], [[quality-gates]], [[known-risks]]

## 2026-05-25: 全画面スクリーンショット時の操作インタラクション改善（上下端ホバー検知の厳格化）

- Context: 全画面スクリーンショット表示時に、マウスを画面中央で少し動かしただけで、またはキー操作やホイール操作をしただけで、×ボタンやフィルムストリップ（ストリップ）が表示されてしまい、没入感を妨げる課題があった。
- Decision:
  1. キーボード操作（左右キー等）やマウスホイール操作時の自動表示を防ぐため、selectViewerIndex や horizontalWheelScroll 内でストリップ（revealFullscreenFilmstrip）を自動表示するのを止め、単にカーソルタイマー（revealFullscreenUi）の延長のみに留める。
  2. マウスのホバー検知領域（on:mouseenter）を上下端のコンポーネント検知divから削除する。
  3. 新たに mousemove イベントでY座標を監視する handleMouseMove を追加し、Y座標が真に画面の最上端（20px以内）または最下端（20px以内）に達したときにのみ×ボタン（revealFullscreenChrome）またはストリップ（revealFullscreenFilmstrip）の表示をトリガーする。
- Rationale: トリガーは画面の極端な上下端（20px以内）でのマウス移動のみに限定しつつ、表示された後は既存のmouseleave（上端112px、下端はストリップを覆うサイズ）や1.6秒自動非表示タイマーをそのまま再利用することで、変更差分を最小限に抑えつつユーザー要求を完璧に満たせる。
- Consequence: 全画面表示時の誤表示が完全に抑止され、没入感の高い画像鑑賞体験が提供される。また、画面端の境界判定ロジックに対してTDDを適用し、ユニットテストを導入した。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-24: アプリ内更新通知は手動実行と demo 確認を前提にする

- Context: 新しい Release を公開した時にアプリ内で知らせたいが、ユーザーの明示操作なしにダウンロード、インストール、再起動が走ると既存作業やゲーム管理を妨げる。
- Decision: 起動時は Tauri updater の `check()` のみを実行し、更新がある場合だけタイトルバーに小さな通知を出す。`downloadAndInstall()` と `relaunch()` は UpdateDialog の「アップデート」押下後だけ呼ぶ。更新前の確認先として GitHub Pages demo と GitHub Release を分けて開く。
- Rationale: Launcherg-Mod はローカルデータとプレイ中の状態を扱うため、更新処理はユーザーがタイミングを選べる必要がある。demo で先に見た目や機能を試せる導線を置くことで、更新判断の材料を増やせる。
- Consequence: updater 用 `.tauri-updater.json` と UI 用 `update-info.json` は release workflow で分けて生成する。Mod版のReleaseタグは `YYYYMMDD` だけを使い、Tauri updater と Windows MSI の制約に合わせて内部バージョンだけ `YY.M.D` に変換する。demo/dev mock では更新通知 UI だけを確認し、実インストールは行わない。
- Links: [[product-context]], [[architecture-map]], [[quality-gates]]

## 2026-05-24: 監視開始前後の未計上プレイ時間を既存累積へ吸収する

- Context: vnite 型の timer / fuzzy time へ近づける残課題として、ゲーム起動からプロセス監視が実際に始まるまでの時間や、最後の監視 tick から終了検知までの端数が、既存のプレイ時間記録に十分反映されない可能性があった。
- Decision: 新しいテーブルや migration は追加せず、ゲーム起動命令時の `Instant` を `GameProcessMonitor` に渡す。監視側は `accounted_until` から現在までの差分を共通の commit 関数で `collection_elements.total_play_time_seconds` と `collection_element_daily_play_times` へ加算する。詳細ページではプレイ時間アイコンから時間詳細パネルを開き、下部の `H:MM` 入力で未記録分の計上と差し引きを行えるようにする。
- Rationale: 現行の総プレイ時間・日別プレイ時間・初回プレイ日時の意味を保ったまま、監視開始前の短い未計上分と終了時の端数を吸収できる。session ledger / reconciliation UI は将来の精密化として残し、今回は既存データモデルの中で改善する。
- Consequence: プレイ時間は起動命令から監視 tick / 終了検知までの実時間により近くなる。プロセス検出自体に失敗した `.lnk` 起動などは、プロセスが特定できないため今後の session ledger / fuzzy bucket の課題として残る。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-23: サイドバー開閉コントロールのタイトルバー移行と完全非表示化

- Context: サイドバー上部にあった「サイドバーを閉じる」ボタンの操作性と、閉じた際にも「最小化された細いサイドバー（MinimalSidebar）」が表示されていて作業領域が狭められるという課題があった。ユーザーはタイトルバーでの開閉管理と、非表示時にはサイドバーが完全に消え去るレイアウトを望んでいた。
- Decision:
  1. `SubHeader.svelte` の閉じるボタンを削除し、`TitleBar.svelte` の左端に `showSidebar` の状態に連動する開閉トグルボタンを新設。タイトルバーをシンプルにするため、アプリのロゴアイコン（Launchergアイコン）を完全に削除し、余白を調整。
  2. `Sidebar.svelte` において、`showSidebar` が `false` のときの幅（width）を `3rem` から `0px` に変更。
  3. `MinimalSidebar` コンポーネントおよびその描画を `Sidebar.svelte` から完全に排除。
  4. 非表示時に境界線が残らないように、`border-r-1px` クラスの適用を `$showSidebar` が `true` の場合のみに制限。
  5. 開閉がもたつかずキビキビと動作するよう、`Sidebar.svelte` から transition-all クラスおよび transition:fly アニメーションを削除して「瞬間的な移行」に変更。
- Rationale: タイトルバーに開閉トグルがあることで、画面全体が広々と使えるようになり、かつサイドバーを完全に畳んで非表示にする（width: 0px）ことでグリッドレイアウト（`grid-cols-[min-content_1fr]`）によってメインコンテンツエリアが画面全体（左端から右端）へとシームレスに拡張される。トグル位置を左端にし、スライドアニメーションを排除して「瞬間的な移行」にすることで、もたつきのない俊敏なUXを実現する。ロゴを削除したことでよりフラットで美しいヘッダー構成になった。
- Consequence: ミニマムサイドバーが廃止され、サイドバー非表示時は完全に画面をフル活用できるようになる。
- Links: [[architecture-map]], [[quality-gates]]��に境界線が残らないように、`border-r-1px` クラスの適用を `$showSidebar` が `true` の場合のみに制限。
  5. 開閉がもたつかずキビキビと動作するよう、`Sidebar.svelte` から transition-all クラスおよび transition:fly アニメーションを削除して「瞬間的な移行」に変更。
- Rationale: タイトルバーに開閉トグルがあることで、画面全体が広々と使えるようになり、かつサイドバーを完全に畳んで非表示にする（width: 0px）ことでグリッドレイアウト（`grid-cols-[min-content_1fr]`）によってメインコンテンツエリアが画面全体（左端から右端）へとシームレスに拡張される。トグル位置を左端にし、スライドアニメーションを排除して「瞬間的な移行」にすることで、もたつきのない俊敏なUXを実現する。
- Consequence: ミニマムサイドバーが廃止され、サイドバー非表示時は完全に画面をフル活用できるようになる。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-23: グローバルな title 属性の自動 Tippy 化とテーマ追従型グラスモフィズムツールチップの導入

- Context: アプリ全体でHTML標準の `title` 属性によるホバー表示が多用されており、これがブラウザ標準の「白地に黒文字」で表示され、アプリ全体のダーク/ライトテーマに合っていなかった。また既存のTippy.js（ButtonBase.svelte）もテーマ指定がなく、表示テーマに統合されていなかった。
- Decision: MutationObserverとTippy.jsのデリゲーションを組み合わせた `setupGlobalTooltips` ユーティリティを `App.svelte` で一元的に導入する。これにより、動的追加・更新される要素も含めて `title` 属性を自動で `data-tippy-content` に移行してブラウザ標準の表示を無効化し、テーマ追従型の美しいTippyツールチップに置き換える。さらに、ツールチップのデザインを角丸、影、および `backdrop-filter` による半透明のグラスモフィズム効果を取り入れたプレミアムな外観へ美化する。
- Rationale: 各コンポーネントの `title` 属性を一つ一つ修正すると差分が膨大になりデグレの危険性があるのに対し、グローバルに属性変更をフックして移行するアプローチは安全かつロータッチで完璧に要求を満たせる。TDDアプローチに沿って移行関数とテストを先に開発した。
- Consequence: すべてのホバー表示が自動的にテーマ（ダーク/ライト）に合わせた美しいデザインに統一され、マウスポインタの移動時の遅延（delay）設定によりUIのプレミアム感が向上する。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-23: Steam パス由来のサムネイル候補と 5 状態 play status を migration なしで導入する

- Context: vnite は Steam など複数データソースと 5 状態の play status を持つ。一方、Launcherg-Mod は ErogameScape 由来 thumbnail と `play_status` INTEGER の 3 状態 UI が中心だった。ユーザー要望として Steam パスの場合の特殊処理、特にゲーム追加時 thumbnail が追加された。
- Decision: Steam インストールパスは `steamapps/common/<install dir>` と近傍の `appmanifest_*.acf` から AppID を推定し、Steam `.url` は `steam://rungameid/<appid>` から推定する。新規追加時の thumbnail 候補は Steam 画像を先に試し、失敗時に既存 ErogameScape thumbnail へフォールバックする。play status は DB migration を追加せず、既存 INTEGER 列で `未プレイ / プレイ中 / クリア済み / 複数進行 / 棚上げ` を UI 全体に通す。
- Rationale: Steam 固有の画像はパスから AppID を取れる場合に品質が高く、追加時だけ既存 thumbnail 保存経路へ候補を渡せば schema を増やさずに改善できる。play status は CHECK 制約がなく、まず UI と既存列の意味を広げるだけなら小さく始められる。
- Consequence: 新状態を選ぶと既存 `play_status` 列に 3/4 が保存されるため、将来は履歴・text enum・bulk migration の設計を検討する。timer ledger / fuzzy time / media asset table / crop-WebP pipeline は別フェーズに残す。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-22: vnite 参照の第一段階は UI-only の受け皿作りに限定する

- Context: vnite 比較レポートでは詳細ページの Header / Tabs / Poster taxonomy と、将来の media / timer / status 拡張が提案されている。一方、今回の作業は既存データと挙動を壊さない安全な第一段階に限定されている。
- Decision: 第一段階では `WorkLayout` と既存 command を維持し、詳細ページを Overview / Record / Memo / Screenshot の常時表示セクションへ整理する。ライブラリカードとサイドバーは既存データだけで見た目と可読性を改善する。DB、migration、SQL、保存データ、play time、play status 値は変更しない。
- Rationale: 情報設計の受け皿を先に作ることで、将来の media asset、play session、extended status を小さく追加しやすくしつつ、現行ユーザーのデータと起動・メモ・スクリーンショット導線を保護できる。
- Consequence: 今回の UI は既存フィールドの再配置に留まる。Background picker、crop/WebP、5状態 play status、playtime reconciliation、save management は別フェーズで DB 変更を含めて判断する。
- Links: [[product-context]], [[architecture-map]], [[quality-gates]]

## 2026-05-21: 初プレイ日時は実プレイ時間の初回記録時に保存する

- Context: ヒートマップやプレイ履歴の振り返りで、登録日やインストール日とは別に「初めて遊んだ日」が必要になった。
- Decision: `collection_elements.first_play_at` を追加し、プレイ時間を実際に加算できた最初の監視タイミングで、値が `NULL` の場合だけ保存する。最終プレイ日時は既存の `last_play_at` を使い続ける。
- Rationale: 起動ボタン押下や起動失敗を初プレイ扱いせず、計測されたプレイ実績に基づく日時として扱える。
- Consequence: 将来のヒートマップでは初プレイ日と最終プレイ日をマーカー表示できる。既存ユーザーの `first_play_at` は、次回以降の実プレイ時から埋まる。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-21: 日別プレイ時間はゲーム別の累積テーブルへ記録する

- Context: GitHub風ヒートマップを後から実装できるよう、既存の総プレイ時間とは別に、日ごと・ゲームごとのプレイ時間が必要になった。
- Decision: `collection_element_daily_play_times` を追加し、`collection_element_id` と `play_date` を主キーにして `play_time_seconds` を累積する。既存の `collection_elements.total_play_time_seconds` は互換性のため維持する。
- Rationale: ヒートマップは日付単位で集計するため、起動セッション履歴を後から推測するより、監視ループの計測時点で日別集計を保存する方が単純で壊れにくい。
- Consequence: 将来の表示実装では日付範囲とゲームIDでこのテーブルを読むだけでよい。日付を跨ぐ短い計測区間はローカル日付ごとに分割して記録する。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-21: SEO初期対応は紹介ページのメタ情報と最小sitemapに限定する

- Context: Launcherg-Mod の公開ページを、日本語ユーザーがノベルゲーム、非Steamゲーム、プレイ時間、スクリーンショット管理の悩みで探す文脈に合わせたい。ただし現状の公開構成は GitHub Pages 上の hash SPA であり、通常URLの機能別ページを増やすにはルーティングと生成方式の判断が必要だった。
- Decision: 初回対応では `Landing.svelte` の title / description / 主要本文を自然な範囲で補強し、初期HTMLに description と OGP / Twitter Card の基本メタを置く。demo build ではトップURLだけを含む `sitemap.xml` と、sitemapを示す `robots.txt` を生成する。機能別ページや `/launcherg-difference` などの新規URLは今回は追加しない。
- Rationale: Google向けの裏技ではなく、ページ内容を人間と検索エンジンの両方に正しく伝える最小差分にするため。hash SPAのまま悩み系ページを量産すると、URLと本文の対応が弱くなり、レビュー範囲も広がる。
- Consequence: 既存UIとdemo導線はほぼ維持される。検索意図別ページ、OGP画像、Search Console設定は次回以降の手動タスクとして残る。
- Links: [[product-context]], [[architecture-map]], [[quality-gates]]

## 2026-05-21: デモ環境のゲーム追加機能（手動・自動）を実際のアプリの見た目でダミー化する

- Context: 公開デモ（demoビルド）において、手動追加画面と自動追加画面が以前は実際のアプリの見た目と異なっていた（自動追加は警告文のみの画面、手動追加は対応していなかった）。ユーザーから「実際のアプリの見た目を体験できるようにしたいが、実際の登録やファイル選択などの処理は停止し、ボタンが押せるだけの張りぼてにする」という要望があった。
- Decision: デモビルド環境（isDemoBuild）において、手動追加（ImportManually）および自動追加（ImportAutomatically）のUIを実際のアプリと統一した。各入力フィールド（Input, InputFilePath, InputPath）、追加/削除ボタン、オプション選択（Checkbox）に disabled 属性を適用し操作を制限した。インポートボタンは活性化された状態にしてクリック可能にし、クリックされた際には実際のリクエストは送らず「demo ではゲーム登録はできません」等の警告トーストを表示してダイアログを閉じるようにした。
- Rationale: デモサイトであっても本来のアプリのUIや導線を見せることで、利用イメージを直感的に掴んでもらえるようにしつつ、ブラウザ上では実行できないデスクトップ依存処理（ファイルダイアログ、自動スキャンなど）を安全にダミー化するため。
- Consequence: デモ環境でもアプリ本来のゲーム追加フローの見た目が体験可能になる。実際に追加は行われず警告トーストが表示される。
- Links: [[product-context]], [[app-inventory]], [[quality-gates]]

## 2026-05-21: 自動起動のWindows登録はリリースビルドのみで行う

- Context: PC起動時の自動起動で、アプリの代わりにターミナル（黒い画面）が開かれる問題が再発していた。調査の結果、開発時（`tauri dev`等）にアプリを起動すると、自動起動処理が `current_exe()`（デバッグパス）をWindowsのRunレジストリに上書きしてしまっていた。デバッグビルドのEXEはコンソールを表示するため、PC再起動時にターミナルが開かれていた。
- Decision: `main.rs` の自動起動設定（`enable()` と `ensure_windows_autostart_entry()`）を `#[cfg(all(desktop, not(debug_assertions)))]` でガードし、リリースビルドのみで実行するようにした。
- Rationale: 開発環境の実行で本番の自動起動エントリが壊されるのを防ぐため。
- Consequence: 開発時にレジストリが上書きされなくなり、インストール先パスが正しく維持される。
- Links: [[architecture-map]], [[known-risks]]

## 2026-05-19: 紹介ページは demo Pages の SPA ルートとして公開する

- Context: GitHub Pages で公開中の demo に加えて、アプリ概要、スクリーンショット、ダウンロード導線をまとめた紹介ページも公開したい。
- Decision: 別サイトや生成物コミットではなく、既存の demo ビルド内で Pages ルートの `#/` を紹介ページにし、通常の demo アプリは `#/demo` へ移す。互換用に `#/landing` でも紹介ページを表示する。
- Rationale: 既存の Pages Actions、mock Tauri 環境、静的ビルドをそのまま使えるため、公開経路を増やさずに紹介ページと体験 demo を共存できる。
- Consequence: 紹介ページの共有 URL は GitHub Pages のルート URL になる。体験 demo は同じ URL に `#/demo` を付けて案内する。画像はリポジトリの `images` 配下を raw GitHub URL で参照する。ダウンロード導線は GitHub API で最新 Release の `.zip` asset を解決し、失敗時だけ既知の最新 zip URL にフォールバックする。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-19: 紐づけ補正は汎用化せず既知パス単位に限定する

- Context: `サクラノ詩` の短いフォルダ名が派生作へ寄る問題に対して、副題前の主題部を一般優先する補正を入れると、他タイトルへ広く影響するリスクがあった。
- Decision: タイトル構造の一般ルールではなく、確認済みのメーカー/フォルダ組み合わせだけを強制IDとして扱う。対象は `枕/サクラノ詩 -> 4529` と `nekoneko/すみれ -> 20178`。
- Rationale: 自動紐づけは誤検知の影響が大きいため、根拠のある実フォルダ差分だけを狭く補正する方が安全。
- Consequence: 同種の誤検知は個別に追加する必要がある。demo プレビューでは、同じフォルダ内に高信頼の紐づけがある場合、そのフォルダ内の確認待ち/候補なし行を出さない。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-18: demo のフォルダ紐づけ判定は通常自動追加と同じ手順にそろえる

- Context: 公開 demo の「フォルダ紐づけを試す」は、通常アプリの自動追加と同じ精度を確認するための入口である。ファイルごとに候補を出すだけだと、通常自動追加で行っている同一ゲームIDの重複解決を再現できない。
- Decision: demo 側の TypeScript mock に、Rust 側の `get_game_candidates_by_exe_path` 相当の正規化、除外語、エンジン名スキップ、距離計算、閾値判定を移植する。さらに `get_map_of_one_filepath_per_game` 相当の重複解決を通し、同じゲームは1つのパスだけ表示する。
- Rationale: ユーザーが確認したいのは相手の実フォルダで通常自動追加した場合の紐づけ精度であり、demo 独自の簡易ロジックでは判断材料にならない。
- Consequence: demo の判定結果は通常自動追加に近い結果になる。候補集合も手元アプリDBの `all_game_caches` から更新する。ブラウザでは Windows の `.lnk` 実体解決やアイコン抽出は完全再現できないため、登録処理は引き続き停止し、判定結果だけを表示する。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-18: demo のゲーム追加は登録ではなく紐づけ確認に限定する

- Context: ブラウザ公開 demo では実ファイルアイコンの取得が安定せず、実アプリと同じ品質でゲーム登録を再現できない。
- Decision: demo ビルドではタイトルバーのゲーム追加とドラッグ&ドロップ登録を無効化し、ホームにフォルダ選択からゲーム紐づけ結果だけを確認するパネルを置く。
- Rationale: 登録後の表示が実アプリと異なる状態になるより、demo で確実に価値を確認できる「所持フォルダからどれだけ紐づくか」に絞る。
- Consequence: demo ではライブラリへゲームを追加できない。通常アプリでは従来どおり自動/手動追加を使える。
- Links: [[product-context]], [[quality-gates]]

## 2026-05-18: demo の公開用画像とHTMLキャッシュはリポジトリに含める

- Context: GitHub Pages は GitHub Actions 上で `npm run build-demo` を実行する。ローカルでは `public/demo-images` と `public/demo-data` が存在していたが、`.gitignore` 対象だったため Actions 環境には渡らず、公開ページで画像が 404 になった。
- Decision: `public/demo-images` と `public/demo-data` は demo のソース資産として Git 管理対象にする。生成物である `docs/demo` は引き続き ignore する。
- Rationale: demo は静的サイトなので、ビルド時に必要な画像とHTMLキャッシュをリポジトリから再現できる必要がある。通常アプリビルドでは別途 demo 資産を除外しているため、アプリ配布物の肥大化は避けられる。
- Consequence: リポジトリサイズは約12MB増える。画像を更新した場合は `public/demo-images` / `public/demo-data` もコミット対象になる。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-18: demo は GitHub Pages に Actions ビルドで公開する

- Context: demo は `npm run build-demo` で `docs/demo` に生成される静的サイトだが、生成物は `.gitignore` 対象でコミットしない運用にしている。
- Decision: `main` への push と手動実行で GitHub Actions が demo をビルドし、`docs/demo` を GitHub Pages artifact として公開する。
- Rationale: 無料で公開でき、生成物をリポジトリに含めず、最新コミットのソースから毎回同じ手順で配信物を作れる。
- Consequence: GitHub 側で Pages の Source を GitHub Actions に設定する必要がある。公開URLは GitHub Pages の deployment 結果に従う。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-18: 通常アプリビルドからデモ用静的資産を除外する

- Context: `public/demo-images` と `public/demo-data` はデモサイトでは必要だが、Vite の通常ビルドでは `dist` にコピーされ、Tauri の `frontendDist` としてアプリ配布物に同梱されていた。
- Decision: 通常の `vite.config.ts` にビルド後削除プラグインを追加し、`dist/demo-images` と `dist/demo-data` をアプリ同梱対象から外す。`vite.demo.config.ts` は変更せず、デモビルドでは従来どおり public 資産を使う。
- Rationale: `publicDir` 全体を無効にすると `icon.png` や `images/dummy_thumbnail.svg` など通常アプリで参照する資産まで壊れるため、デモ専用ディレクトリだけを削るのが最小変更。
- Consequence: アプリ配布物は約9MB以上軽くなり、デモページ用の大きい画像とHTMLは通常アプリに入らない。デモ用静的資産を増やす場合は、この除外対象に入るディレクトリへ置く。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-18: 自動起動Runエントリの大小文字削除を止める

- Context: PC起動時の自動起動が再び失敗していた。`HKCU\Software\Microsoft\Windows\CurrentVersion\Run` を確認すると `Launcherg` の登録が残っておらず、補修処理は `Launcherg` を書いた直後に旧名 `launcherg` を削除していた。
- Decision: 旧名削除の判定は大文字小文字を無視して比較し、`Launcherg` と `launcherg` を同一名として扱う。
- Rationale: Windowsレジストリの値名は実質的に大文字小文字を区別しないため、文字列比較だけで別名扱いすると、補修直後のRunエントリ自身を削除してしまう。
- Consequence: 次回アプリ起動時に `"exe path" --autostart` のRunエントリが残る。旧名が本当に別名だった場合のみ削除する。
- Links: [[architecture-map]], [[known-risks]]

## 2026-05-11: 探す画面を一時撤去し、自動起動のWindows登録を補修する

- Context: 探す画面は外部データ連携と検索UXの不安定さが残り、いったんアプリ本体から外す判断になった。PC起動時の自動起動はWindowsのRun登録で実行ファイルパスが引用符なしになると、空白を含むインストールパスで失敗する。
- Decision: 探すのルート、タイトルバー導線、専用ビュー、Discovery専用DBコマンドを削除する。自動起動は `Launcherg` 名のRunエントリを `"exe path" --autostart` 形式で補修し、旧来の小文字エントリを片付ける。
- Rationale: 不安定な探索面を残して他の導線を巻き込むより、確実に動く本体機能へ戻す。自動起動はユーザー操作ではなく起動時に必ず必要なため、Tauri pluginの登録結果をWindows向けに明示補正する。
- Consequence: 探す機能のコードと専用キャッシュDBはアプリから参照されない。将来復帰する場合は、外部データ探索を別ブランチ相当のまとまりで再設計する。
- Links: [[architecture-map]], [[known-risks]]

## 2026-05-11: 探す検索はローカル候補と軽量ErogameScape SQLを併用する

- Context: `all_game_caches` は軽いがタイトルとサムネイルだけを持つため、ブランド検索や統計条件を正しく判定できない。
- Decision: 条件の軽いタイトル検索では `all_game_caches` を先に使い、不足時やブランド・統計条件がある時だけErogameScape SQLへ進む。SQLは検索条件で詳細を直接SELECTせず、まず作品IDだけを取得してからID指定の詳細SQLで補完する。
- Rationale: ErogameScapeのSQLフォームには実行コスト制限があり、タイトル・読み・ブランド・統計を一度に結合した検索は拒否される。ID検索に分けると正確さを保ちながら負荷と失敗率を下げられる。
- Consequence: ブランド名や評価条件を含む検索ではErogameScape SQLに依存する。ローカル候補とSQL候補の両方を発見用DBにキャッシュし、同じ検索での再アクセスを抑える。
- Links: [[architecture-map]], [[known-risks]]

## 2026-05-11: 探す画面の画像フォールバックにVNDBカバーを使う

- Context: ErogameScape由来のDMM/DLsite/駿河屋/getchu画像だけでは、一部作品でサムネイルが表示できない。
- Decision: まずErogameScape由来画像を試し、失敗時にgetchu画像へ切り替え、それも失敗した場合だけVNDBのカバー画像を検索して使う。VNDB結果は発見用キャッシュへ30日保存する。
- Rationale: 探索ランキングと検索はErogameScape中心のまま保ちつつ、画像欠落だけをVNDBで補完できる。VNDB APIは既存の直列キューとタイムアウトを使うため、失敗画像のたびに無制限アクセスしない。
- Consequence: 画像フォールバックの品質はVNDBの検索一致に依存する。タイトル一致、発売日、ブランド名で候補を選ぶが、誤一致を避けるため一致スコアが取れない結果は採用しない。
- Links: [[architecture-map]], [[known-risks]]

Launcherg-Modの継続改善に影響する判断を残す場所です。量が増えたら、判断ごとに別ノートへ分割します。

## 2026-05-10: 発売予定と最近のおすすめはErogameScape本体ページを優先する

- Context: 「探す」画面の発売予定と最近のおすすめは、SQLだけで再現するとErogameScape本体で見える並びや運用上の意図からずれる可能性がある。
- Decision: 発売予定は `before_reserve.php`、最近のおすすめは `toukei_osusume_saikin.php` から作品IDの並びを取得し、そのIDだけを固定SQLで補完する。ページ取得や解析が失敗した場合は従来の固定SQLへフォールバックする。
- Rationale: 本体ページの並びを尊重しつつ、アプリ側ではサムネイル、所持済み判定、気になる状態、100点表示など同じカードUIに正規化できる。ページ取得もSQLiteキャッシュと直列キューを通すため、アクセス頻度を抑えられる。
- Consequence: 棚の品質はErogameScape本体ページのHTML構造にも依存する。壊れた場合でもSQLフォールバックで空表示を避ける。
- Links: [[architecture-map]], [[known-risks]]

## 2026-05-10: VNDB発見機能は専用DBとクライアント側キューで守る

- Context: VNDB APIを使った「探す」画面では、外部API結果、ユーザーの発見状態、既存ライブラリDBの互換性を同時に扱う。
- Decision: 発見用の状態と検索キャッシュは `launcherg_discovery.db3` に分離し、外部データ取得はフロント側の直列キューと短いタイムアウト、SQLiteキャッシュで制御する。既存メインDBのmigration runnerは、将来の新規migrationがあるDBを旧アプリで開いた場合でもmissing migrationで即失敗しない設定にする。
- Rationale: 購入ストアではなく探索画面なので、多少古いキャッシュを許容してもUXは成立する。一方で既存ライブラリDBは起動不能リスクが高いため、発見機能の拡張データを混ぜない方が安全。日本向け探索ではVNDBよりErogameScapeの統計が適している。
- Consequence: 発見画面の検索精度はErogameScapeの公開SQLフォームとキャッシュ済み統計テーブルに依存する。任意SQLは受け取らず、固定クエリだけを組み立てる。将来、発見状態をメインDBへ統合する場合は互換性方針を再検討する。
- Links: [[architecture-map]], [[known-risks]]

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
