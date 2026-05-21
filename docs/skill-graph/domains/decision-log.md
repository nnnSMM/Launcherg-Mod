---
id: decision-log
title: Decision Log
type: log
status: active
updated: 2026-05-21
links:
  - launcherg-improvement-moc
  - template-decision-record
  - source-skill-graphs-note
---

# Decision Log

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
