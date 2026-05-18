---
id: decision-log
title: Decision Log
type: log
status: active
updated: 2026-05-18
links:
  - launcherg-improvement-moc
  - template-decision-record
  - source-skill-graphs-note
---

# Decision Log

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
