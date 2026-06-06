---
id: remote-play-hub
title: Remote Play Hub
type: improvement
status: proposed
updated: 2026-06-06
links:
  - product-context
  - idea-pipeline
  - quality-gates
  - idea-bank
  - decision-log
  - remote-play-companion-ux-research
  - mobile-companion-service-blueprint
---

# Remote Play Hub

全体サービス設計は [[mobile-companion-service-blueprint]] に分ける。このノートでは、主にスマホプレイ、補助コントローラー、リモートプレイ周辺の体験を扱う。

## Problem

PCのノベルゲームをiPhone/iPadで遊びたいが、ゲーム起動、配信アプリ起動、接続確認、メモ、スクリーンショット、プレイ時間記録が分散すると使いづらい。Launcherg-Mod側で開始から記録までをまとめ、スマホ側の手動操作を減らしたい。

## Product Shape

Launcherg-Modは映像配信エンジンではなく、スマホプレイの司令塔、補助コントローラー、持ち出し用ライブラリになる。映像と通常入力はMoonlight/SunshineまたはSteam Linkへ任せ、Launcherg-Modはゲーム起動、状態確認、接続案内、メモ同期、スクリーンショット挿入、一時停止、ノベルゲーム向けの小さな操作マクロ、PC補助外のゲーム一覧とスクリーンショット閲覧を扱う。

## Target Flow

これはスマホプレイ導線まで育てた後の将来像。最初のMVP実装範囲ではない。

1. ゲーム詳細で「スマホでプレイ」を押す。
2. Launcherg-Modが対象ゲームを起動し、プレイ時間記録を開始する。
3. 配信ホストの準備状態を確認し、未設定なら最短手順を表示する。
4. iPhoneでQRを読み取り、Launcherg Remote画面を開く。
5. 映像はMoonlight/SunshineまたはSteam Linkで見る。
6. Launcherg Remoteからメモ、スクリーンショット挿入、一時停止、撮影マクロを操作する。
7. 終了後、記録とメモと画像はPC側のLauncherg-Modに残る。

## MVP Flow

これは最初に実装判断する流れ。配信ホスト確認、スマホでプレイ開始、Library、Gallery、撮影マクロは含めない。

1. PC版の対象ゲーム画面から既存SkyWay接続ページを開く。
2. PC版が短命sessionId、allowedGameId、scope、TTL付きのQRを発行する。
3. iPhoneでQRを読み取り、公開HTTPS PWAのLauncherg Remote画面を開く。
4. スマホ側に対象ゲーム、接続状態、Pause状態、許可された操作を表示する。
5. スマホからPause切替または通常スクリーンショットだけを実行する。
6. 撮影中、成功、失敗をスマホ側に返し、直前スクショの小さなプレビューだけを表示する。
7. TTL切れ、対象ゲーム終了、PC側QR閉鎖、別gameId、scope外操作は拒否し、再接続へ戻す。

## Smallest Useful Change

最初は「スマホでプレイ」ウィザードではなく、既存のSkyWay接続ページを安全化し、Pause状態表示、Pause切替、通常スクリーンショットの成功フィードバックだけを検証する。配信開始、Library、Gallery、文字消しマクロは後続フェーズに分ける。

## MVP Scope

- QR発行時に短命sessionId、allowedGameId、scope、TTLを作る。
- 受信メッセージはsessionId、gameId、scope、TTLを検証してから処理する。
- iPhone側操作: Pause状態表示、Pause切替、通常スクリーンショット、撮影中/成功/失敗表示。
- Screenshot: 既存のメモ画像保存経路だけを使う。Capture Galleryには登録しない。
- State: 対象ゲーム、接続状態、Pause状態、撮影結果を返す軽い状態応答を定義する。
- 制限: 1台のPC、1台のスマホ、1ゲーム、マクロなし、Galleryなし、Libraryなし、外出先接続なし。

## Companion Controller

### Primary Buttons

- Pause: プレイ時間計測を一時停止/再開する。既存の `toggle_pause_tracking` 相当をスマホから呼ぶ。
- Screenshot: PCの主ディスプレイ全体を撮影し、メモ画像として保存する。ゲーム起動プロセスIDに依存する `save_screenshot_by_pid` はスマホ補助スクショの正規経路にしない。
- Status: Pause状態、接続状態、直前スクショを表示する。これらはメモへ自動追記しない。

MVP後に、Clean Screenshot、Delayed Screenshot、Quick Memoを順に追加する。

### Screenshot Presets

- Normal: 入力せずに即撮影する。
- Hide Text: 右クリック、Space、H、Ctrlなど、作品ごとの文字消しキーを実行してから撮影する。
- Delayed: 1秒/3秒後に撮影する。演出や表情差分を待つ用途。
- Retake: 直前スクショのプレビューから撮り直す。削除は誤操作を避けるため確認付きにする。

MVPではNormalだけを扱う。Hide Text、Delayed、Retakeは信頼境界とアクティブセッション状態が固まった後に追加する。

### Companion Display

- Elapsed Time: プレイ開始からの経過時間と今日のプレイ時間を表示する。メモには自動挿入しない。
- Last Screenshot Preview: 直前のスクショだけスマホに小さく返し、撮影成功を確認する。
- Screenshot Caption: 必要な時だけ撮影直後に一言コメントを付ける。
- Private Note: 感想や攻略メモを通常メモとして残す。

### Control Safety

- 入力マクロは作品ごとのプリセットにする。右クリックで文字が消えない作品や、右クリックがメニュー表示になる作品を避けるため。
- マクロ実行前に対象ゲームのウィンドウが見つかることを確認する。
- 文字消しマクロは `前操作`、`待機ms`、`撮影`、`後操作` の順で決定的に実行する。
- 最初は右クリック/キー入力の少数プリセットだけ許可し、任意スクリプト実行はしない。
- PauseやScreenshotも、sessionId、allowedGameId、scope、TTLを検証してから実行する。
- 現行の `startProcessMap` はフロント状態なので、再接続やリロードで失われうる。MVP前に対象プロセス状態を確認できる応答を定義する。

## Out Of Scope

- Launcherg-Mod本体による低遅延映像配信。
- GPUエンコード、音声配信、入力仮想化の自前実装。
- 外出先接続、VPN、ポート開放の自動化。
- Sunshine設定ファイルの自動書き換え。
- App Store配布前提のネイティブiOS実装。
- 任意コマンド実行や自由入力マクロ。

## UX Requirements

- 入口は常にLauncherg-Modのゲーム詳細に置く。
- ユーザーに複数アプリを往復させる場合も、次に押す場所を1画面に集約する。
- 未設定、未起動、接続切れはウィザード内で復帰できる文言にする。
- 映像配信アプリ側で必要な初回ペアリングは隠さず、初回だけの手順として扱う。
- ノベルゲーム向けに、画面下部のメモ、スクショ、Pause操作を優先する。
- スマホ側の主要操作は片手で押せる大きなボタンにする。
- 誤操作が重い操作は長押しまたは確認付きにする。通常スクショやメモ追加は即時でよい。

## Experience Principles

- Flow First: プレイ中の視線はPC画面に残す。スマホ側は見なくても押せる主操作と、見た時だけ役立つ状態表示に分ける。
- One-Handed Control: Pause、通常スクショ、文字消しスクショは親指で届く下部に固定する。重要操作は44px相当以上のタップ領域にする。
- Immediate Feedback: ボタンを押した瞬間に押下状態、送信中、成功、失敗を表示する。1秒を超える処理は「撮影中」「接続確認中」のように明示する。
- Quiet Status: 経過時間、Pause状態、接続状態、直前スクショは常時見えるが、主操作を邪魔しない小さな表示に留める。
- Setup Is The Product: Sunshine/Moonlight/Steam Linkの初回設定は外部アプリ任せに見せない。Launcherg-Mod側で「今何が足りないか」「次にどこを押すか」を示す。
- Game-Specific Safety: 文字消し操作は作品ごとに保存し、既定値を押し付けない。初回だけテスト撮影で確認してから使う。
- No Accidental Damage: 削除、終了、マクロ設定変更は確認付きにする。撮影やPauseのような軽い操作は即時実行にする。
- Recover In Place: 接続切れ、撮影失敗、対象ウィンドウ喪失が起きても、PC画面に戻らずスマホ側で再接続や再試行へ進める。

## Experience Model

### Library Layer

- PC補助中でない時は、スマホ側を「ライブラリ」として開く。
- ホームは最近遊んだゲーム、プレイ中、未プレイ、お気に入り、最近撮ったスクリーンショットを小さく並べる。
- ゲーム一覧は検索、プレイ状況、最近プレイ、プレイ時間、未インストールで絞り込めるようにする。
- ゲーム詳細はサムネイル、プレイ状況、総プレイ時間、最近のスクショ、メモの読み取りを中心にする。
- PCが同一LANで起動中なら「このゲームをPCで開く」「スマホでプレイを開始」を出す。PCが見つからない時は閲覧だけにする。

### Gallery Layer

- スクリーンショットは「全体タイムライン」「ゲーム別」「最近撮影」「お気に入り」に分ける。
- まずサムネイルだけを送り、フルサイズは開いた時だけ取得する。
- 直近のプレイセッションごとにまとまりを作ると、後で振り返りやすい。
- 共有/保存は明示操作に限定する。自動クラウド同期はしない。
- 削除や一括操作はスマホ側MVPには入れず、最初は閲覧と保存だけにする。

### Offline Cache Layer

- スマホには最近のゲーム一覧、状態、サムネイル、低解像度スクショだけをキャッシュする。
- メモ本文とフルサイズ画像はプライバシーが強いため、初期設定では端末キャッシュしない。
- キャッシュ画面には最終同期時刻と「PC未接続」を明示する。
- PC未接続時の操作は、検索、閲覧、次に遊ぶ候補の確認に限定する。

### Setup Layer

- 初回は「配信アプリを設定してください」ではなく、Launcherg-Mod内で現在足りないものを1つずつ見せる。
- Sunshine/Moonlight/Steam Linkは選択式にし、選んだ方式だけの手順を出す。
- 成功条件は「iPhoneでゲーム画面が見える」「Launcherg Remoteが接続済み」「Pause/スクショのテストが通った」の3つにする。
- 2回目以降はウィザードを飛ばし、前回成功した方式で即開始する。

### Play Layer

- 下部: MVPではPauseと通常スクショを固定する。後続で文字消しスクショ、遅延スクショを追加する。
- 上部: 接続状態、経過時間、Pause状態、現在の作品名だけを小さく表示する。
- 中央: 直前スクショプレビューとメモ。どちらも折りたためる。
- メモ入力中でも下部のPauseとスクショは隠さない。
- 横向きiPhoneでは左にメモ、右に操作ボタン。縦向きでは上から状態、プレビュー/メモ、操作ボタン。

### Recovery Layer

- 接続切れ: 再接続ボタン、QR再表示、PC側状態確認を同じ画面に出す。
- 撮影失敗: 対象ウィンドウなし、権限/キャプチャ失敗、ゲーム未起動を分けて出す。
- 文字消し失敗: 「戻す」だけを大きく出し、次回からそのプリセットを無効にできる。
- ゲーム終了検知: 記録を止める、メモを保存する、スクショを確認する、の順に閉じる。

## Competitive Lessons

- Steam Remote Play: ゲームごとのタッチ設定は有効。ただしLauncherg-Modでは自由配置ではなく、作品別の撮影プリセットに限定する。
- Moonlight/Sunshine: 映像配信品質は任せる。Launcherg-Modはペアリングや開始前チェックの不安を減らす。
- Touch Portal/Deckboard: マクロパッドの強みは1タップ実行と視覚フィードバック。Launcherg-Modでは汎用マクロ化せず、ノベルゲーム中の少数操作へ絞る。
- GameGlass: PWAで補助パネルを始められる。iOSネイティブ化の前に、Web/SkyWay画面で体験を固める。
- Unified Remote: Wake-on-LANやSiri/URIは後続の起動導線として有望だが、MVPには入れない。
- PlayStation App / Xbox App: キャプチャ閲覧とライブラリ管理は、遠隔プレイとは別のスマホ価値になる。Launcherg-ModでもPC補助外の主画面として扱う。
- Steam Remote Downloads: PC側が起動している時だけ遠隔管理できる前提はLauncherg-Modにも合う。PC未接続時は閲覧中心にする。

## Research Notes

- W3C/WCAG 2.5.8は、隣接する小さなターゲットの誤操作を避けるため、最低24px相当のターゲットまたは十分な間隔を求めている。頻繁に使うスマホ操作はこれより大きく扱う。
- WCAG 2.5.5の解説では、特にタッチでは44px相当以上のターゲットが推奨され、頻繁に使う操作、取り消しにくい操作、画面端の操作では大きいターゲットが重要とされる。
- AppleのUI Design Tipsでも、指で正確にタップできるよう44pt x 44pt以上のコントロールが示されている。
- NN/gのVisibility of System Statusは、操作が受け付けられたかを即時に伝えることが不確実性と連打を減らすとしている。
- NN/gの応答時間目安では、0.1秒は即時反応、1秒は思考の流れが途切れにくい上限、1秒超では作業中表示が必要とされる。
- MoonlightはiOS/iPadOS向けクライアントを提供し、SunshineはMoonlight向けホストとして低遅延、ハードウェアエンコード、Web UIでの設定とペアリングを持つ。Launcherg-Modはここを再実装せず、初回設定とプレイ中補助の体験を整える。

## Evaluation

- Impact: 高。PC前に座らずにプレイを継続でき、Launcherg-Modの記録価値も残る。
- Effort: 中。最初は既存SkyWay拡張に絞れるが、sessionId/scope/TTL、対象gameId、プロセス状態確認を入れる必要がある。
- Confidence: 中。Moonlight/SunshineやSteam Linkに任せれば映像品質は担保しやすい。
- Risk: 中。現行SkyWayの信頼境界、メモ画像保存とGallery保存の混同、外部ツール依存、同一LAN検出で詰まりやすい。

## Verification

- `npm run check`
- `npm run test:run`
- PC実機でゲーム詳細から「スマホでプレイ」を開き、既存ゲームを起動できること。
- iPhoneでQR接続し、Pause状態表示、Pause切替、通常スクリーンショットの成功/失敗表示が動くこと。
- 期限切れsessionId、scope外操作、別gameIdの操作が拒否されること。
- 撮影結果がメモ画像経路に保存され、Capture Galleryに出る前提を置いていないこと。
- iPhoneのホーム画面から公開HTTPS PWAを起動し、QR接続後に同じControllerへ戻れること。
- Sunshine/MoonlightまたはSteam Linkで同じゲーム画面を表示できること。

## Follow-up

- Sunshineインストール済み検出。
- Moonlight/Sunshine向け起動プリセット。
- Wake-on-LAN。
- iPad向け横画面UI。
- iOSネイティブ companion app 化。
