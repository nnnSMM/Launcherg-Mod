---
id: mobile-companion-service-blueprint
title: Mobile Companion Service Blueprint
type: improvement
status: proposed
updated: 2026-06-06
links:
  - product-context
  - remote-play-hub
  - remote-play-companion-ux-research
  - quality-gates
  - idea-bank
---

# Mobile Companion Service Blueprint

## Service Statement

Launcherg Mobile Companionは、PC上のLauncherg-Modをスマホから見る、始める、補助するためのPWAです。PC版がライブラリの正本、スマホ側PWAが閲覧、撮影補助、プレイ中操作、最近データの持ち出しを担います。

## Roles

- PC版Launcherg-Mod: 正本データ、ゲーム起動、プロセス監視、プレイ時間記録、スクリーンショット保存、外部配信ツールの案内。
- Mobile Companion: ライブラリ閲覧、スクリーンショット閲覧、プレイ中補助、接続状態表示、最近データのキャッシュ。
- Moonlight/Sunshine/Steam Link: 映像配信と通常入力。
- ローカルネットワーク: PCとスマホの同期路。初回はQR、将来はBonjour/自動検出を検討する。

## PWA Delivery Model

Mobile Companionは公開HTTPS originのPWAとして配る。既存のQR遷移先である `https://launcherg.ryoha.moe` を正式なスマホ面に育て、PCローカルHTTPサーバーをiPhoneから直接叩く構成はMVPに入れない。

- App Shell: `manifest.json`、アプリアイコン、`display: standalone`、Service Workerを持つ。Controller画面はネットワーク復帰時にも開けるよう、最低限のシェルだけキャッシュする。
- Control Channel: PC操作はSkyWay data channelで行う。PWAから任意のPCローカルAPIや任意コマンドを直接呼ばない。
- Library Thumbnails: `library_response` では画像本体を送らず、サムネイルのローカルパスだけを返す。PWA側は表示対象を `thumbnail_request` で16件ずつ要求し、PC側は直近のライブラリ応答に含まれる許可済みパスだけを順番にチャンク送信する。全サムネイルを接続直後に一括送信しない。
- QR Payload: `roomId`、`sessionId`、`allowedGameId`、`scopes`、`expiresAt`、`pcName`、起動モードを含める。URLに載せる値は短命で、PC側セッションレジストリに存在する場合だけ有効にする。
- Install UX: iPhoneではPWA内からインストールプロンプトを強制表示できない前提で、Safari共有メニューからホーム画面へ追加する短い手順を出す。iOS 26以降の「Open as Web App」は有効にする前提で案内する。
- Viewport: ホーム画面追加後はアプリ的に扱えるよう、ピンチ/ダブルタップズームを抑止する。通常の縦スクロールは残す。
- Offline: 初期はPWAシェル、接続説明、直近の軽い状態だけをキャッシュする。メモ本文、フルサイズ画像、ローカルパスはキャッシュしない。
- Updates: Service Workerの更新はセッション中に強制適用しない。プレイ中のControllerは現在の接続を維持し、更新は次回起動時に反映する。

## Service Modes

### Pocket Library

PC補助中でない時の主画面。ゲーム一覧、最近遊んだゲーム、プレイ中、未プレイ、お気に入り、最近のスクリーンショットを見せる。PC未接続でも、最後に同期した範囲だけ見られる。

### Session Launcher

スマホから「PCで開く」「スマホでプレイ」を開始する入口。PCが同一LANで見つかった時だけ表示する。ゲーム未インストール、PC未接続、配信未設定の時は実行ボタンではなく復帰手順を出す。

### Companion Controller

プレイ中だけ前面に出る操作面。Pause、通常スクショ、文字消しスクショ、遅延スクショ、クイックメモ、経過時間、直前スクショを扱う。ゲーム一覧や詳細編集は隠し、プレイを邪魔しない。

### Capture Gallery

スクリーンショットを時系列、ゲーム別、最近撮影で見る。初期は閲覧と端末保存だけ。削除、並び替え、一括整理はPC版に残す。

### Recovery Desk

接続切れ、PC未検出、ゲーム終了、撮影失敗、文字消しマクロ失敗を扱う画面。エラー画面で止めず、次の操作を1つに絞る。

## Information Architecture

- Home: 最近遊んだゲーム、次に続ける、最近のスクショ、PC接続状態。
- Library: 検索、プレイ状況、最近プレイ、総プレイ時間、未インストールで絞り込み。
- Game Detail: サムネイル、状態、時間、最近スクショ、メモ閲覧、PCで開く、スマホでプレイ。
- Gallery: 全体タイムライン、ゲーム別、最近撮影、フル表示、端末保存。
- Connect: PC接続、QR読み取り、接続テスト、外部配信ツールの状態。
- Controller: プレイ中だけの専用面。通常タブとは別の一時モードとして扱う。

## Mobile UX Reframe

スマホ版はPC版の縮小コピーにしない。通常時は「見返す・選ぶ・始める」ためのPocket Library、プレイ中だけ「見ずに押せる」Controllerへ切り替える。

- Navigation: Bottom Navは現在 `Home`、`Library`、`Connect` に限定する。`Gallery` はスクリーンショットDB同期が入った段階で追加する。`Controller` はタブではなく、プレイ中のNow Playingバーまたはゲーム詳細の主CTAから開く一時モードにする。
- Home: 接続状態、続きから遊ぶ、最近遊んだゲーム、最近のスクショだけを置く。統計カードや全フィルタはLibraryへ逃がし、最初の画面を軽くする。
- Library: スマホ向けの1カラムカード一覧にする。カードはサムネイル、タイトル2行まで、ブランド、状態、最終プレイ、導入済みだけに絞る。検索は上部固定、フィルタは横スクロールさせず2行グリッドに収める。PC版の表密度を持ち込まない。
- Game Detail: 読み取り専用の作品面にする。状態、時間、最近スクショ、メモプレビューを見せ、操作は `PCで開く`、`スマホでプレイ`、`補助を開く` に絞る。Pause、スクショ、メモ編集を直接置かない。
- Controller: Pause、通常スクショ、文字消しスクショ、クイックメモ、直前結果、接続状態だけを大きく置く。遅延スクショ、作品別プリセットはここから開く二段目のモードにする。
- Gallery: Controllerの撮影結果とは保存経路を混同しない。初期は閲覧、フル表示、端末保存だけ。削除や整理はPC版に残す。
- Recovery: 接続切れ、期限切れ、PC未起動、対象ゲーム終了は専用状態として扱い、次の操作を `再接続`、`QRから開き直す`、`前回同期を見る` のどれか1つに絞る。

重複排除の原則は、ゲーム詳細が「対象を理解して始める場所」、Controllerが「プレイ中にPCへ命令する場所」です。同じPause/スクショ操作を両方に置かない。

## State Model

- Offline: PC未接続。キャッシュ閲覧だけ可能。
- Discovering: PC探索中。QR接続または手動URL入力へ逃がす。
- Paired: PCと認証済み。ライブラリ更新、サムネイル取得が可能。
- Ready: 対象ゲームが起動可能。PCで開く、スマホでプレイが可能。
- Playing: プレイ時間記録中。Controllerを出せる。
- Streaming Assisted: 外部配信ツールで映像を見ながら、Controllerで補助操作中。
- Paused: 記録一時停止中。復帰ボタンを強調する。
- Recovering: 接続、撮影、マクロ、ゲーム終了の問題を解決中。

## Active Session Model

スマホからPause、スクリーンショット、将来のマクロを実行する前に、PC側にアクティブセッションの概念を作る。現行のSkyWay経路は `type` と `gameId` が妥当なら処理へ進むため、このまま操作を増やさない。

- sessionId: QR発行ごとに作る短命ID。
- expiresAt: セッションTTL。期限切れ後の操作は拒否する。
- allowedGameId: 操作できるゲームID。別ゲームIDのmemo/screenshot/pauseは拒否する。
- scopes: `memo:read`, `memo:write`, `screenshot:capture`, `pause:toggle` のように許可操作を分ける。
- activeProcessId: 対象ゲームの現在のPID。フロントの `startProcessMap` だけに依存せず、状態取得APIで確認できるようにする。
- connectionId/deviceId: 再接続時に同じ端末か確認する。再接続でTTLやscopeが伸びないようにする。

最初の実装では、1台のPC、1台のスマホ、1ゲームだけを許可し、複数ゲーム同時操作は扱わない。

## Data Policy

- 正本はPC側SQLiteとPC上の画像ファイル。
- スマホ初期キャッシュは、作品ID、タイトル、サムネイル、プレイ状況、時間、低解像度スクショだけ。
- メモ本文、フルサイズスクリーンショット、ローカルパスは初期設定では端末へ永続保存しない。
- スマホからの破壊的操作はMVPでは持たない。削除、完全同期、DB編集はPC版へ残す。
- 共有/端末保存は明示操作に限定し、自動クラウド同期はしない。

## Screenshot Storage Boundary

リモートスクリーンショットはスマホからPCへ「主ディスプレイ全体を撮る」命令を送り、`game-memos` 配下に画像を保存し、メモへMarkdown画像を挿入する経路です。これはスクリーンショットDBへ登録する `import_screenshot` / `get_game_screenshots` 系のギャラリー経路とは同一ではない。ゲーム起動プロセスIDに依存する `save_screenshot_by_pid` は、スマホ補助スクショの正規経路にしない。

- Controller MVP: 既存経路を使い、撮影成功フィードバックと直前プレビューだけを検証する。Capture Galleryへは表示しない。
- Gallery MVP: screenshot DBを読み取る閲覧機能として別フェーズにする。
- 統合する場合: 「メモだけ」「ギャラリーだけ」「両方」を撮影時に明示するか、新しい `capture_game_screenshot` コマンドで撮影、DB登録、メモ挿入を一貫して扱う。
- 初期既定: プレイ中のスマホ撮影は「メモへ挿入」。ギャラリー保存は後続判断にする。

## Pairing And Trust

- 初回はPC版のQRから接続する。
- QRには短命トークン、PC名、接続先、許可スコープを含める。
- スマホ側で端末名を登録し、PC側で後から解除できる。
- PC未接続時にできること、接続中にできることを明確に分ける。
- ネイティブiOS化した場合は、BonjourによるPC検出とiOSのローカルネットワーク許可を導入候補にする。
- 受信メッセージは `sessionId`、`gameId`、`scope`、`issuedAt` を検証してから処理する。
- セッションを閉じた、TTLが切れた、対象ゲームが終了した、PC側でQRを閉じた場合は、再接続しても操作を拒否する。
- PauseやマクロのようなPC状態を変える操作は、信頼境界が実装されるまで追加しない。

## Target Onboarding

これはPocket LibraryとCapture Galleryまで含めた将来の全体オンボーディング。First MVPでは最近のゲーム一覧や直近スクショの同期を行わない。

1. PC版で「スマホ連携」を開く。
2. スマホでQRを読み取る。
3. スマホに「このPCに接続しました」を表示する。
4. PC版で端末名と許可範囲を確認する。
5. サンプルとして最近のゲーム一覧と直近スクショを同期する。
6. プレイ中補助を使う場合だけ、Pause/スクショのテストを行う。

## MVP Onboarding

これはFirst MVPのオンボーディング。既存SkyWay接続ページの安全化とプレイ中補助だけを確認する。

1. PC版の対象ゲーム画面から既存SkyWay接続ページを開く。
2. PC版が短命sessionId、allowedGameId、scope、TTL付きのQRを発行する。
3. スマホでQRを読み取り、対象ゲーム、接続状態、Pause状態、許可された操作を表示する。
4. スマホからPause切替と通常スクショのテストだけを行う。
5. Library/Gallery同期、最近ゲーム一覧、直近スクショ取得、撮影マクロは後続フェーズとして表示しない。

## First MVP

最初のMVPは、PWA全体のHome/Library/Galleryではなく、公開HTTPS PWA上の既存SkyWay接続ページを安全化し、プレイ中補助に絞る。

- PC版: QR発行時に短命sessionId、allowedGameId、scope、TTLを持つセッションを作る。
- スマホ側PWA: 既存SkyWay接続ページにPause状態表示、Pause切替、通常スクショの送信中/成功/失敗表示を足す。
- Screenshot: PCの主ディスプレイ全体を撮り、既存のメモ画像経路だけへ保存する。Capture Galleryとは接続しない。
- State: 対象ゲーム、Pause状態、接続状態、撮影結果を取得する軽い状態APIまたはSkyWay応答を定義する。
- 制限: 公開HTTPS PWA/既存SkyWay、1台のPC、1台のスマホ、1ゲーム、マクロなし、Galleryなし、Libraryなし、削除なし、外出先接続なし、PCローカルAPI直叩きなし。

## Growth Path

- Phase 0: PWAシェルスパイク。`https://launcherg.ryoha.moe` でmanifest、Service Worker、ホーム画面追加、QR起動、SkyWay接続をiPhone実機で確認する。
- Phase 1: 公開HTTPS PWA上の既存SkyWay接続ページを安全化し、Pause/通常スクショの補助MVPを作る。
- Phase 2: 読み取り専用Pocket Library。Home、Library、Game DetailだけでGalleryは含めない。
- Phase 3: Capture Gallery。screenshot DBの読み取り、サムネイル、フル表示、端末保存。
- Phase 4: スマホでプレイ開始ウィザード。
- Phase 5: 作品別撮影プリセットと文字消しスクショ。
- Phase 6: Bonjour検出、Wake-on-LAN、iOSネイティブ化。

## Connectivity Spike

PWAは公開HTTPS origin + SkyWayを第一選択にする。PWAからPCローカルAPIを直接叩く方式は、Library/Galleryフェーズに入る前の別スパイクとして扱う。配布形態によってCORS、mixed content、証明書、iOSでの扱いが変わるため、体験設計と実装境界を先に決める。

- Option A: 公開HTTPS PWA + SkyWay。Controller MVPの採用案。HTTPS要件を満たしやすく、既存QR/SkyWay資産に近い。PCローカルAPIへ直接触らない前提にする。
- Option B: PCローカルサーバー配信。LAN内データ取得は自然だが、HTTPS証明書とiOSの扱いが課題。
- Option C: PC版がQRごとに短命Web UIを提供し、SkyWayで操作だけ通す。ローカル配信寄りなので、証明書と更新配布が重くなる。
- 判定基準: iPhone実機でホーム画面追加、PWA単体起動、QR接続、SkyWay状態取得、再接続、Service Worker更新が破綻しないこと。

## Success Criteria

- Controller MVPでは、初回QR接続が2分以内に終わる。
- iPhoneのホーム画面からLauncherg Mobile CompanionをPWAとして起動できる。
- PC画面へ戻らず、スマホからPauseとスクショが成功する。
- 許可されていないgameId、期限切れsessionId、scope外操作が拒否される。
- 通常スクショの保存先がメモ経路であることがUIと設計で混同されない。
- Galleryフェーズでは、スクショ閲覧でサムネイル一覧は軽く、フルサイズは必要時だけ開ける。
- PC未接続時に、できない操作が誤って表示されない。
- スマホ側から削除や破壊的変更をしないため、既存データ破損リスクが増えない。

## Design Risks

- スマホ側に機能を増やしすぎると、PC版と役割が重複する。
- オフラインキャッシュを広げすぎると、メモや画像のプライバシーリスクが上がる。
- 配信ツールの初回設定に失敗すると、Launcherg-Mod側の印象も悪くなる。
- PC未接続時の期待値を誤ると「壊れている」と感じられる。
- PWAで始める場合、ホーム画面追加やストレージ挙動はネイティブアプリより説明が必要になる。
- 既存SkyWayメッセージのまま操作を増やすと、許可していないgameIdや操作を受け付けるリスクがある。
- メモ画像保存とスクリーンショットDB登録を混同すると、スマホで撮った画像がGalleryに出ない、または重複保存される。
