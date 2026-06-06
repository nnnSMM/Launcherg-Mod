---
id: decision-log
title: Decision Log
type: log
status: active
updated: 2026-06-06
links:
  - launcherg-improvement-moc
  - template-decision-record
  - source-skill-graphs-note
---

# Decision Log

## 2026-06-06: Mobile CompanionのQRは補助コントローラー入口にする

- Context: スマホ連携をゲーム詳細やメモ用QRの延長に見せると、ユーザーは「詳細ページの一機能」と認識してしまう。今回の要求では、補助操作は現在PCで開いているゲームへ自動接続し、詳細ページからは補助へ飛べないようにする必要がある。
- Decision: PC側の共通QRは `mode=controller` の補助コントローラー入口にし、QRから特定ゲームを選ばせない。スマホPWAはPCの `control_status.activeGameId` を受け取り、Launcherg-Modで追跡中のゲームを自動選択する。スマホのゲーム詳細から補助操作CTAは外し、詳細は閲覧、補助は専用Controllerに分ける。
- Rationale: 補助操作は「いまプレイ中のゲーム」に対する一時的な操作なので、ライブラリ選択や詳細画面の文脈より、PCの追跡セッションを正にする方が誤操作が少ない。
- Consequence: PCでLauncherg-Modからゲームを起動して追跡セッションができるまでは、スマホ補助画面は待機状態になる。ライブラリ閲覧は残すが、補助開始の主経路ではなくなる。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-06: Pause復帰とスクショプレビューはプレイを邪魔しない

- Context: Pause解除後に元のゲームへフォーカスが戻らず、プレイ再開に余計なクリックが必要だった。また、スマホ/独自ショートカット経由のスクショは保存されたか分かりにくかった。
- Decision: Pause表示時にWindowsの前面ウィンドウを記録し、Pause解除時にそのウィンドウへフォーカスを戻す。スクショプレビューはLauncherg-Modのスマホ撮影と独自スクショショートカットだけで表示し、Windows標準スクショ監視やメモ挿入用スクショでは出さない。
- Rationale: Pauseとスクショはプレイ中の補助操作なので、成功確認は小さく出しつつ、ゲームへの入力復帰を妨げないことを優先する。
- Consequence: フォーカス復帰はWindowsの前面制御に依存する。OSが復帰を拒否した場合でも、Pause解除自体は続行する。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-06: Mobile Companionの操作判定はRustの追跡セッションを正にする

- Context: スマホからPause/スクショができない報告があった。スクショの起動中判定をPCフロントの `startProcessMap` に依存させると、QR再接続、PWA再起動、PC側画面リロードで状態が消え、Rust側では追跡中でもスマホ操作が失敗する。
- Decision: 現在追跡中のgameId/processIdは `PauseManager` の追跡セッションとしてRust側に保持する。Mobile Companionのスクショは、要求gameIdがRustの追跡セッションと一致する場合だけ実行する。スマホ側へ返すControl StatusもRustの追跡状態を使う。
- Rationale: PauseもスクショもPCアプリの追跡セッションに紐づく操作なので、ブラウザ状態ではなくTauri/Rust側の状態を信頼境界にする方が再接続に強い。
- Consequence: Launcherg-Modでゲームを起動し、プロセス検出が完了するまではスマホ操作は失敗する。将来、手動起動ゲームを拾う場合はRust側の追跡セッション作成APIを追加する。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-06: Mobile Companionの補助スクショは通常スクショとして保存する

- Context: スマホ補助スクショで `対象ゲームの起動プロセスが見つかりません` が出ていた。原因はSkyWayの `take_screenshot` が `startProcessMap` から対象ゲームのPIDを取得し、`save_screenshot_by_pid` へ渡す実装だったこと。その後、ユーザーから「メモ挿入ではなく、起動中ゲームが分かっている場合だけアプリ側の通常スクショとして保存する」と訂正があった。
- Decision: Mobile Companionの通常スクショ/文字消しスクショは、対象ゲームが `startProcessMap` 上で起動中として記録されている場合だけ実行する。撮影対象はPCの主ディスプレイ全体にし、撮影結果はスクリーンショットDBへ登録する。メモへMarkdown画像は挿入しない。スマホ補助スクショでは撮影自体に `save_screenshot_by_pid` を使わない。
- Rationale: アプリ側のスクショ一覧と同じ扱いにすると、スマホで撮った画像を後からGallery/ゲーム詳細で自然に見返せる。起動中ゲームが不明な場合は保存先の紐づけが曖昧になるため、失敗として返す方が安全。
- Consequence: マルチモニター環境では初期実装は主ディスプレイを撮る。将来、撮影対象モニター選択や全モニター合成が必要になったらControllerの撮影設定として追加する。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-06: Mobile CompanionサムネイルはPWA側のオンデマンド要求にする

- Context: PC側が `library_response` の直後に全サムネイルを一括チャンク送信すると、ゲーム件数が多いライブラリでdata channelへ負荷が集中し、一部サムネイルが届かない。送信前に送信済み扱いすると、失敗した画像も同じ接続中に再送されない。
- Decision: `library_response` はゲーム情報とサムネイルパスだけを返す。PWA側はHome、Library、Detailで表示対象になったゲームのサムネイルを `thumbnail_request` で16件ずつ要求する。PC側は直近のライブラリ応答に含まれる許可済みパスだけを順番に送信し、成功後に送信済みとして扱う。
- Rationale: 一括送信を避けると、スマホ一覧の表示に必要な画像から優先的に取り込める。PC側で許可済みパスに限定すれば、PWAから任意ローカルファイルを要求される境界も避けられる。
- Consequence: サムネイル欠落時はPWA側が一定間隔で再要求する。Galleryやフルサイズ画像は同じ仕組みに載せる前に、表示用サムネイルと保存対象画像の境界を別途決める。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-06: Mobile Companion UIは閲覧タブとプレイ中Controllerを分離する

- Context: PWAの接続は成立したが、一覧がスマホ向け密度になっておらず、Game DetailとControllerの両方にPause、スクショ、メモ操作があり、役割が重複している。今後Gallery、文字消しスクショ、スマホでプレイ開始、ネイティブ化を足すと、通常閲覧とプレイ中操作がさらに混ざる。
- Decision: Mobile Companionの通常ナビゲーションは現在 `Home`、`Library`、`Connect` に分け、`Gallery` はスクリーンショットDB同期が入った段階で追加する。`Controller` はタブではなくプレイ中だけ開く一時モードにする。Game Detailは読み取り専用と開始導線に絞り、PauseやスクショはControllerへ集約する。Expo/React Native化は、PWAで体験を固めた後、Bonjour、ローカルネットワーク権限、ネイティブ通知、端末保存などが必要になった段階で再判断する。
- Rationale: スマホで使う主価値は、非プレイ時のPocket Library/Galleryと、プレイ中の補助Controllerで文脈が違う。同じ操作を詳細画面と操作画面に置くと、どちらが正規の操作面か分からなくなる。現在の実装はSvelte/Vite PWAなので、Expoへ早期移行するとReact Nativeへの再実装コストが大きく、まだ固まっていないUIの手戻りが増える。
- Consequence: 次の実装は新機能追加ではなく、MobileCompanion画面の情報設計を先に直す。Bottom NavからControllerを外し、Now Playingバーまたは詳細CTAからだけ開く。Libraryはサムネイル付き1カラムカードへ作り直し、横スクロールする絞り込みを避ける。Game Detailから直接Pause/スクショを削除し、文字消しスクショはControllerへ集約する。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-companion-ux-research]]

## 2026-06-06: Mobile Companionのホーム画面起動はQR接続情報を保持する

- Context: 静的manifestの `start_url` が `companion.html` 固定だと、iPhoneのホーム画面から起動した時にQR URLの `roomId` と `authToken` が落ちる。さらに公開PWAから `https://launcherg.ryoha.moe/connect` へ直接tokenを取り直す経路はブラウザ上で `Failed to fetch` になり、`roomId` だけでは再接続できない。PC側もdev再読み込みで `roomId` が変わると、ホーム画面アイコンの接続先とPC側のroomがずれる。
- Decision: 静的manifestから `start_url` を外し、ホーム画面追加時のQR URLを起動URLとして残せるようにする。QR画面で差し替える動的manifestの `start_url` には `roomId` とQR発行時の `authToken` を含める。PC側のMobile Companion roomIdはlocalStorageに保存し、アプリ再読み込み後も同じroomへ戻す。
- Rationale: 現在の配布構成では、PWA単体が新しいSkyWay tokenを発行できないため、即時再起動の体験を成立させるにはQRで渡された接続情報を起動URLに残す必要がある。静的manifestの固定 `start_url` を消すことで、iOSが動的manifestを採用しない場合でも現在URLを保持する余地を作る。
- Consequence: 既に壊れたURLで追加済みのホーム画面アイコンはOS側に古い起動URLが固定されているため、削除してQRから追加し直す必要がある。`authToken` が期限切れになった後の再接続は引き続き失敗しうるため、将来的には公開PWAから安全にtokenを再発行できるCORS/session APIへ置き換える。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-05: Mobile Companion操作MVPはHTTPS PWA上で検証する

- Context: スマホ実機でPCの `npm run tauri dev` にLAN HTTP接続すると、ブラウザが安全なコンテキストと見なさず、SkyWay/WebRTCに必要な `navigator.mediaDevices` が無効になって白画面になった。
- Decision: スマホ側の実接続検証はGitHub PagesなどのHTTPS PWAで行う。LAN HTTPで開いた場合はSkyWayを読み込まず、接続不可であることを画面に表示する。操作MVPとして、Pause状態取得、Pause切替、通常スクリーンショットの成功/失敗応答をSkyWay data channelで返す。
- Rationale: HTTPS配布を前提にすればiOS/Androidのブラウザ制約を避けられ、白画面ではなく診断可能な状態にできる。スクショとPauseはPC状態を変えるため、スマホ側に結果を返してユーザーが操作完了を判断できる必要がある。
- Consequence: `npm run tauri dev` はPC側ホストの検証には使えるが、スマホ側をLAN HTTPで本接続検証する用途には使わない。実機確認はPagesへデプロイ後に行う。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-05: Mobile Companionは接続後にPCライブラリ全体を同期する

- Context: スマホ連携QRが特定ゲームまたは既存メモ画面に見えると、ユーザーは「アプリ連携」ではなく「メモ用QR」と認識してしまう。PC補助をしていない時にもスマホ側でゲーム一覧を見たいという要求がある。
- Decision: QRはアプリ単位の連携入口にし、スマホPWAは接続後に `library_request` を送り、PC側は登録済みゲーム一覧を `library_response` で返す。対象ゲームはスマホ側の一覧から選択する。
- Rationale: 全ゲーム一覧を最初に出すことで、PWAが単なるメモ同期画面ではなくLauncherg-Modのスマホ面として成立する。補助操作は選択中ゲームに対して行えばよく、ゲーム詳細にQRを置く必要もなくなる。
- Consequence: 初期の一覧にはローカルファイル画像を載せない。サムネイルやスクショ一覧は、PWAで安全に配れる画像経路を決めてから追加する。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-05: Mobile Companion入口はゲーム詳細ではなくアプリ全体に置く

- Context: 既存のQRはゲーム詳細アクション内にあり、ユーザーには「元のメモ用QR」と区別しにくかった。Mobile CompanionはLauncherg-Modアプリと連携する入口なので、個別詳細の一操作として置くと発見性と意味づけが弱くなる。
- Decision: Mobile CompanionのQR入口はタイトルバーの共通操作として一か所に置く。QRは特定ゲームに固定せず、接続後にスマホ側でゲーム一覧から対象を選ぶ。
- Rationale: 入口を一か所に固定すると、将来Pause、スクリーンショット、表示状態、Library/Galleryを足しても「スマホ連携はここ」という認知を維持できる。ゲーム詳細側はPlay/設定/お気に入りなど、そのゲーム自体の操作に絞れる。
- Consequence: ホームや設定からも同じタイトルバー入口でQRを開ける。ゲーム詳細側のQRは廃止し、特定ゲームの補助操作はスマホ側で対象を選択してから行う。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]]

## 2026-06-05: Mobile Companionは公開HTTPS PWAとして提供する

- Context: Mobile CompanionをPWAにする方針が決まった。既存実装ではQRの遷移先が `https://launcherg.ryoha.moe` で、SkyWay roomId/gameIdを渡す形になっている。一方、PWAからPCローカルHTTP APIを直接叩く構成はHTTPS、Service Worker、CORS、証明書、認可の難度が上がる。
- Decision: Mobile Companionは公開HTTPS originのPWAとして提供する。既定のPWA配布先はGitHub Pagesの `https://nnnsmm.github.io/Launcherg-Mod/` に揃え、QRにはPC側が取得したSkyWay `authToken` と `roomId` を含める。Controller MVPではPWAからPCローカルAPIを直接呼ばず、既存SkyWay data channelを短命sessionId、allowedGameId、scope、TTLで安全化して使う。manifest、アイコン、Service Worker、ホーム画面追加導線を正式な配布要件にする。
- Rationale: PWAはApp Store配布なしでiPhoneから試しやすい。PWA配布先とSkyWay認証APIを別originにするとCORSやプリフライトで詰まるため、PC側で取得済みの短命SkyWayトークンをQRに載せ、スマホ側はQRだけでSkyWay roomへ参加できるようにする。Service WorkerやinstallabilityはHTTPS前提のため、ローカルサーバー配信より公開HTTPS PWAの方が初期検証の不確実性が小さい。
- Consequence: iOSネイティブ機能、Bonjour自動検出、Wake-on-LAN、同一LAN読み取りAPIは後続に回す。iPhoneではインストール操作をPWA内で強制できないため、Safari共有メニューからホーム画面へ追加する案内をUXに含める。GitHub Pages上のQR入口は `companion.html` で受け、内部では既存の hash SPA ルート `#/companion` へ遷移させる。オフラインキャッシュはシェルと軽い状態に限定し、メモ本文やフルサイズ画像は初期キャッシュしない。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]], [[remote-play-companion-ux-research]]

## 2026-06-05: Mobile CompanionのMVPはSkyWay安全化とPause/通常スクショに絞る

- Context: Mobile Companion構想で、PWAのHome/Library/Game Detail/Gallery/ConnectとControllerを同時にFirst MVPへ含めると、Growth Pathとの境界が矛盾し、接続方式、認可、スクショ保存先、状態管理の未解決事項を抱えたまま実装へ進むリスクがあった。
- Decision: 最初のMVPは既存SkyWay接続ページの安全化に限定する。短命sessionId、allowedGameId、operation scope、TTLを導入し、Pause状態表示、Pause切替、通常スクリーンショットの成功/失敗フィードバックだけを検証する。Library PWA、Capture Gallery、スマホでプレイ開始ウィザード、文字消しマクロは後続フェーズへ分ける。
- Rationale: 現行SkyWayメッセージは `type` と `gameId` の構造検証が中心で、操作権限やセッション失効を扱っていない。さらに現行の `save_screenshot_by_pid` はメモ画像保存経路であり、screenshot DBのGallery経路とは別であるため、まず信頼境界と保存先を固定する必要がある。
- Consequence: 初回実装の価値は小さくなるが、PauseやスクリーンショットというPC状態を変える操作を安全に増やせる。スマホ側のライブラリ閲覧やGalleryは、PWA/API接続方式スパイクとスクショ保存方針を終えてから実装する。
- Links: [[mobile-companion-service-blueprint]], [[remote-play-hub]], [[remote-play-companion-ux-research]]

## 2026-06-05: スマホプレイはLauncherg-Modを司令塔にし、映像配信は既存ツールへ任せる

- Context: iPhone/iPadでPC上のノベルゲームを遊ぶ構想では、できるだけLauncherg-Mod側の操作で完結したい一方、低遅延映像配信、音声、入力転送まで自前実装すると範囲が大きくなりすぎる。
- Decision: Launcherg-Modは「スマホでプレイ」の入口、ゲーム起動、状態確認、接続案内、メモ、スクリーンショット、一時停止を担当する。映像配信と入力転送はMoonlight/SunshineまたはSteam Linkを使う前提にする。
- Rationale: 既存のSkyWay連携でメモ同期とスクリーンショット指示の土台があり、Launcherg-Modの価値はプレイ記録と管理にある。映像配信は専用ツールの方が品質と保守性が高い。
- Consequence: 初回MVPは同一LANと既存ツール連携に限定する。将来のiOSネイティブ化は、Remote Play Hubの操作面が固まってから検討する。
- Links: [[remote-play-hub]], [[product-context]], [[quality-gates]]

## 2026-06-05: 失敗表示は決定的な分類レイヤーでユーザー向け文言とログを分ける

- Context: Clauge参考の検討で、Windows統合・インポート・ショートカット・更新・スクリーンショットの失敗時に、生エラー文字列だけではユーザーもCodexも次の確認に進みにくいことが分かった。
- Decision: フロントエンドに小さな `errors.ts` を置き、既知の失敗を権限、パスなし、ショートカット形式、使用中、ネットワーク、DBなどに分類する。Toastには次の行動が分かる文言を出し、元エラーは文脈付きでログに残す。ショートカット登録はRust側で保存前に検証し、登録失敗時は旧状態へ戻す。
- Rationale: 失敗理由の分類はAI要約ではなく決定的なコードで行う方が再現性が高い。既存のToastやTauri command境界に薄く差し込めるため、設定画面や更新機構の作り直しを避けられる。
- Consequence: 代表的な失敗はユーザー向けに読みやすくなるが、未知のエラーは引き続きログ確認が必要。分類パターンは実機報告に合わせて追加する。
- Links: [[clauge-reference-integration]], [[known-risks]], [[quality-gates]]

## 2026-06-02: スクリーンショット候補取得は互換APIを残して停止する

- Context: スクリーンショット候補は外部サイトから候補画像URLを取得して `game_screenshot_caches` に保存していたが、現行UIでは候補を表示する経路がなく、取り込み後の外部アクセスと未使用チャンクだけが残っていた。
- Decision: 現アプリから候補取得の入口を削除し、未使用の候補解析モジュールとプレビューコンポーネントを削除する。一方で既存DBテーブル、マイグレーション、Tauriコマンドは互換用に残し、既存ユーザーDBや古い呼び出しを壊さない。
- Rationale: 表示価値が出ていない外部取得は軽量化・省データ化の対象だが、DBスキーマやコマンドを同時に消すと過去バージョンからの移行リスクが上がる。まずは副作用の入口を閉じるのが安全。
- Consequence: 新規/変更インポート後に候補画像URLの外部取得は走らない。過去に保存された `game_screenshot_caches` は未使用データとして残るが、起動や通常操作を妨げない。
- Links: [[architecture-map]], [[quality-gates]], [[known-risks]]

## 2026-06-02: 初期ロード最適化は画面単位・機能単位の遅延読み込みを優先する
- Context: 安定化後の追加調査で、demo の初期 JS に route helper 経由の mock Tauri core / demo catalog が混入し、初期 CSS には Google WebFont 生成 CSS が大量に含まれていた。通常アプリ側でも Memo / SkyWay / markdown / updater / work registration など、初期画面で不要な機能が早く読み込まれていた。
- Decision: 初期表示に必須ではない画面・機能・外部ライブラリは、Svelte route の `asyncComponent` と dynamic import で遅延読み込みする。画面判定などの純粋関数は Tauri API や demo data を top-level import しない。WebFont は外部生成 CSS ではなく OS フォント中心の fallback を使う。
- Rationale: 起動直後に使わない依存を分離すると、クラッシュ面では初期化失敗の影響範囲が狭まり、性能面では初期転送量・解析量・CSS 量が下がる。demo では特に「見るだけ」の初期表示が重いと体験が悪化するため、データチャンクは必要な画面遷移後に読むべきである。
- Consequence: 初期 JS/CSS は大幅に小さくなった。一方で Memo editor の EasyMDE 本体は機能利用時の大きな遅延チャンクとして残るため、将来さらに軽くするなら editor ライブラリ選定か専用 chunk 戦略を別途検討する。
- Links: [[architecture-map]], [[quality-gates]], [[known-risks]]

## 2026-06-02: 安定化リファクタリングは保存値復旧・購読寿命・panic削減を優先する

- Context: 「全体的に動作が不安定」として、壊さない前提で広くデバッグとリファクタリングを行う依頼があった。初期検証では主要ゲートは通ったが、テスト後のSimpleBar/jsdomノイズ、ESLint設定不備、localStorage破損時の同期クラッシュ、Svelte storeの購読増殖、Rust/Tauriの起動経路 `unwrap()` が見つかった。
- Decision: UIやDBスキーマの意味を変えず、まずは保存済み設定の復旧、コンポーネント寿命に沿った購読管理、SimpleBar破棄、Rust側のpanic削減、検査コマンドの復旧を優先する。大きなUX変更や機能追加は今回の安定化作業に混ぜない。
- Rationale: 不安定さの原因は再現が難しいことが多いため、時間経過・画面遷移・壊れたローカル状態・OS連携失敗で落ちない土台を先に固める方が安全。既存のテスト・型検査・ビルドが通っている状態を保ちながら、回帰テストを追加できる箇所から固定する。
- Consequence: 起動不能やリーク起因の重さは減る。DB接続不能などアプリ継続が難しい致命的失敗は引き続き起動失敗として扱うが、ログと原因特定はしやすくなる。Windows実機のトレイ・ショートカット・スクリーンショット挙動は別途手動QAが必要。
- Links: [[architecture-map]], [[quality-gates]], [[known-risks]]

## 2026-06-01: dev起動では実updater確認を走らせない

- Context: `npm run tauri dev` でスクリーンショット撮影中に、GitHub Release を見に行く実更新通知が反応して作業状態を妨げた。
- Decision: `import.meta.env.DEV` または public demo build では実 updater の `check()` を呼ばず、明示的な `mockUpdate` 指定がある場合だけ更新通知UIの確認を許可する。
- Rationale: 開発・撮影・demo確認では実インストール経路に触れる必要がなく、更新UIの検証は mock で再現できる。production では従来どおり起動時の確認だけ行い、インストールはユーザー操作後に限定する。
- Consequence: `npm run tauri dev` で予期せず更新通知が出なくなる。更新通知UIを確認したい時は `?mockUpdate=1` または mock 用 localStorage を使う。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-31: デモ環境における works.ensureRegisteredStories モック不足による画面遷移不具合の解消

- Context: アプリ起動時に未取得のストーリー詳細を自動取得する機能として works.ensureRegisteredStories() が App.svelte の初期化処理に導入されたが、デモ環境（isPublicDemoBuild = true）で使用されるモック src/mock/works.ts に同メソッドが定義されていなかった。このため、紹介ページ（ランディングページ）からデモページへ遷移する際に works.ensureRegisteredStories is not a function という同期的 TypeError が発生し、Svelte のリアクティブ更新サイクルがクラッシュして画面が切り替わらない不具合が起きていた。
- Decision: TDDスタイルに基づき、まずモック works が ensureRegisteredStories メソッドを安全に呼び出せる（例外を投げず Promise を解決する）ことを確認するテスト src/mock/works.test.ts を作成し、その後 src/mock/works.ts に Promise.resolve() を返すダミーの同メソッドを実装して不整合を解消した。
- Rationale: デモ環境では実アプリのような外部からのストーリー事前取得は不要であるため、ダミー実装として定義だけを持たせることで、App.svelte 側の同期的例外の発生を防ぎ、画面遷移がクラッシュせず正常に行われるようにするため。
- Consequence: デモ環境での紹介ページとデモページの双方向の遷移が、再読み込みなしでスムーズに機能するようになる。
- Links: [[launcherg-improvement-moc]], [[quality-gates]]

## 2026-05-31: updaterリリースはdefaultマニフェストと現在鍵へ固定する

- Context: 20260524 から 20260531 への自動更新で、旧アプリが参照するマニフェスト名と署名検証用公開鍵が現在のリリース成果物とずれ、更新検知または署名検証で失敗するリスクがあった。
- Decision: updater の公開マニフェスト名は `default.tauri-updater.json` に統一し、Tauri 設定の `pubkey` は現在の署名秘密鍵に対応する公開鍵へ揃える。Release workflow はアセット指定をスラッシュ形式へ寄せ、既存 Release が残っていても `allowUpdates` とアセット置換で再リリースできるようにする。アプリが `/releases/latest/download/...` を参照するため、updater 対象の Release は prerelease ではなく通常 Release として公開する。
- Rationale: 先頭ドットの隠しファイル名や手動アップロードに依存すると、CI/CD と GitHub Release の間で名前ずれが起きやすい。既存 Release 更新を workflow に任せることで、タグ再投入だけで署名済み成果物とマニフェストを一貫して差し替えられる。GitHub の latest は prerelease を対象にしないため、prerelease のままだとタグ直指定は成功してもアプリの latest endpoint が 404 になる。
- Consequence: 20260524 のホットフィックス再リリースと 20260531 以降のリリースで、手動アセット調整なしに同じ updater 経路を使える。署名鍵を変更する場合は、旧版から次版へ渡るための互換リリースを先に用意する必要がある。
- Links: [[architecture-map]], [[quality-gates]], [[maintenance-routine]]

## 2026-05-31: 通常時100%不透明および詳細時完全クリア透過・ぼかしなしのサイドバー背景設計

- Context: プレミアムなグラスモーフィズム没入体験をサイドバーに統合する際、当初は通常時および詳細時にすりガラス効果（backdrop-blur-xl）を適用していた。しかし、背景色を完全に透明（0%）に設定しても、すりガラス of ぼかし効果自体がブラウザのレンダリング上で白濁したグレーの霧（ベール）を自動合成してしまい、サイドバー全体に白黒グレーの色が残って見えてしまう課題があった。また、通常時は背景画像が存在しないため、すりガラスにする意味がなく、文字の可読性が低下する課題もあった。
- Decision: 通常表示時は、背景画像がないため元の安定したソリッドな「100%不透明背景（bg-bg-secondary）」に戻し、余分なすりガラスぼかし（backdrop-blur）も適用しない。詳細表示時のみ、サイドバーコンテナの背景色を完全に透明（bg-transparent）にし、かつ、すりガラスのぼかし効果も完全にオフ（backdrop-blur-none）に切り替える設計とした。以前「6%」や「10%」に調整していたSubHeaderやSearchなどの内部パネル背景クラスはすべて削除し、元の正常な標準表示（bg-bg-primary/20 および 30）に戻した。
- Rationale: 通常時は100%不透明にすることで安定した高い可読性を保つ。一方、詳細画面に入った際には、背面の画像自体がすでに「64px」の超強力ぼかしによって極上の色彩グラデーションになっているため、サイドバー上のぼかし効果を完全にオフ（無効化）にすることで、余計な白濁ノイズを100%完全に消し去ることができる。これにより、背面画像の美しい色彩グラデーションが一切の濁りなく極限までクリアに透過する、洗練されたプレミアムな色彩没入体験が実現できるため。
- Consequence: 通常時の可読性と、詳細画面での色彩透過・没入感が双方ともに完璧に向上する。
- Links: [[launcherg-improvement-moc]], [[quality-gates]]

## 2026-05-31: 詳細ページ背景ぼかし強度の64pxへの引き上げ

- Context: プレミアムなグラスモーフィズム没入体験をより際立たせるため、詳細ページの背面に表示されるカバー画像のぼかし量が 32px では元の画像がやや視認されすぎ、透過する周辺UIの文字との競合や質感の一体感が不十分になる懸念があった。
- Decision: 詳細ページ表示時の最背面カバー画像に適用される CSS ぼかし（filter: blur）の強さを 32px から 64px に引き上げる。
- Rationale: ぼかし量を 64px に引き上げることで、画像自体の輪郭が程よく融解し、作品ごとの個別の色彩テーマが画面全体に自然かつ柔らかにグラデーションとして溶け込む。これにより、周辺のグラスモーフィズムUIとの一体感が向上し、高品質でプレミアムな没入感の高いビジュアル体験が実現できる。
- Consequence: 最背面の画像は滑らかなグラデーション状になり、テキストの可読性と没入感が格段に向上する。
- Links: [[launcherg-improvement-moc]], [[quality-gates]]

## 2026-05-31: プレイ状況を4分類へ整理し中断を追加する

- Context: プレイ状況に「複数進行」と「棚上げ」があり、運用上の意味が近く選択肢が増えすぎていた。ユーザーはこの2つを消し、代わりに「中断」を入れることを求めた。
- Decision: UI上のプレイ状況は「未プレイ」「プレイ中」「クリア済み」「中断」の4つに整理する。保存値は既存互換のため `3` を「中断」として使い、過去の `4`（旧棚上げ）は表示・絞り込み・統計では「中断」に含める。
- Rationale: DBマイグレーションなしで既存データを見失わず、今後の新規設定値は4分類に統一できるため。
- Consequence: 旧「複数進行」の保存値は「中断」として表示される。旧「棚上げ」の保存値も中断として集計され、新規に棚上げ値を設定するUIはなくなる。
- Links: [[product-context]], [[quality-gates]]

## 2026-05-31: ヒートマップ内では時間量を文字表示しない

- Context: プレイ時間の計上・補正により、合計プレイ時間とヒートマップ内の時間表記がそれぞれ変動し、ユーザーが「確定した実績値」として読み取りにくくなる懸念があった。
- Decision: 統計ページと詳細ページで共通利用しているヒートマップから、合計時間、最大日別時間、セルホバーの日別時間表示を削除する。代わりに記録日数、最新記録日、最長連続記録日数を表示し、セルのラベルは「記録あり/なし」に留める。
- Rationale: ヒートマップは日ごとの活動有無と密度を眺める部品として扱い、正確な時間量は他のプレイ時間表示に任せることで、同じ画面内で時間の意味が重複・競合するのを避けるため。
- Consequence: 色の濃淡は従来どおり記録量に基づくが、文字としての時間値は出さない。今後ヒートマップに追加する情報も、時間量そのものではなく活動日・最新日・連続性のような俯瞰指標を優先する。
- Links: [[product-context]], [[quality-gates]]

## 2026-05-31: ホームの全体アクティビティを統計ページへ分離する

- Context: ホーム画面に全ゲーム合算のアクティビティヒートマップが置かれていたが、ホームは最近の履歴と登録ゲーム一覧の入口として使う画面であり、ライブラリ全体の振り返り情報を増やすには表示密度と役割が合わなくなっていた。
- Decision: 全体アクティビティは `/stats` の専用統計ページへ移し、タイトルバーから固定タブとして開けるようにする。統計ページには合算ヒートマップに加えて、登録数、総プレイ時間、今月/今年のプレイ、連続記録、クリア率、お気に入り数、プレイ状態分布、曜日リズム、累計プレイ時間上位、登録年、発売年代、最近登録したゲームを配置する。
- Rationale: ホームは起動直後の行動導線を軽く保ち、統計ページは「眺める」「振り返る」情報を広い画面で扱えるように役割を分けるため。既存の日別プレイ時間と `CollectionElement` だけで算出できる情報に限定し、DB変更なしで安全に増やせる。
- Consequence: 全体ヒートマップの取得負荷はホーム表示時には発生せず、統計ページを開いた時だけ全ゲーム分の日別記録を集計する。今後スクリーンショット数や月別推移などを足す場合も、統計ページに集約する。
- Links: [[product-context]], [[architecture-map]], [[quality-gates]]

## 2026-05-30: 詳細ページ記録下部への大型プレイヒートマップ追加

- Context: 詳細ページの「記録」タブでは累計プレイ時間やコミュニティ統計は確認できるが、日ごとのプレイ傾向を俯瞰できなかった。GitHub の contribution graph のように、過去1年の活動密度を一目で見たいという要望があった。
- Decision: 記録タブの一番下に大型のプレイヒートマップを追加する。データは `collection_element_daily_play_times` から日別プレイ時間を取得し、demo では既存の合計プレイ時間・最終プレイ日から決定的に日別データを生成する。ヒートマップの基準色はサムネイル画像を Canvas に読み込み、RGB 各チャンネルの二乗平均平方根（RMS）で算出した色を使う。
- Rationale: 記録タブの末尾に大きく配置することで、既存の統計情報を読んだ後に活動履歴を広い面積で確認できる。色をサムネイル由来にすることで作品ごとの印象と記録表示が自然につながり、固定色の汎用グラフより詳細ページ全体のテーマに馴染む。
- Consequence: 実アプリでは日別プレイ時間テーブルが表示元になる。demo では実データベースを持たないため、同じUIが確認できるようにモック側で日別レコードを保持・更新する。
- Links: [[architecture-map]], [[quality-gates]]

## 2026-05-30: ゲーム詳細画面の背景処理リファインと周辺 UI のグラスモーフィズム統合

- Context: ノベルゲームや非Steamゲームなどの管理における没入感（UX）向上のため、詳細画面の背景デザインをさらにプレミアムなものへとリファインする要望があった。従来の Canvas にじみ描画や SVG フィルターを用いた背景処理を廃止し、よりモダンなグラスモーフィズム表現（すりガラス効果）をサイドバーやタイトルバー、タブバーを含む周辺 UI 全体に連動させ、ゲームごとのテーマカラーが美しく溶け込むデザインの統合が必要であった。
- Decision:
  1. 重たい Canvas 描画、ResizeObserver、SVG フィルター（ink-water）、背景色グラデーションを WorkLayout.svelte から完全に削除し、詳細画面の基本レイアウトを bg-transparent 化した。
  2. 詳細画面表示中（isWorkDetailRoute）は、ゲームのカバー画像を最背面の固定背景レイヤー（App.svelte）に「強力にぼかした状態（blur-3xl、opacity-85、scale-105）」で配置し、ゲームごとの個別のテーマカラーを画面全体に優しく演出する設計とした。
  3. 詳細画面表示中は、タイトルバー（TitleBar.svelte）の背景（bg-bg-primary/92）や、サイドバー（Sidebar.svelte）の背景（bg-bg-secondary）を透明（bg-transparent）に切り替え、既存の backdrop-blur-xl との相乗効果で背面のぼかし画像が美しく透けるようにした。
  4. タブバー（ATabList.svelte）の右側空白領域（bg-bg-disabled）や、タブ自体（ATab.svelte）の背景も詳細画面表示中は透過させ、ホバー時に極薄の白（hover:bg-white/10）をあてることで視認性とプレミアムなグラスモーフィズム質感を両立させた。
  5. グラス情報領域（GlassInfo.svelte）から背景色（bg-bg-primary/28）を取り除き、色なし透明（bg-transparent）に変更して完全に無色透明なブラー効果を実現した。
  6. 詳細ルートの判定および画像URL生成ロジックに対して、t_wada氏のTDDスタイルを適用し、テストコード（routeHelper.test.ts）を先に設計・パスさせる形で開発を推進した。
- Rationale: Canvas のにじみ処理を排してピュアな透過レイアウトにすることでパフォーマンス負荷が劇的に軽減される。同時に、アプリ最背面でゲーム画像を極限までぼかして固定配置し、周辺 UI をグラス透過させることで、画面全体がそのゲームの色彩テーマで優しく染まる、macOS や Windows Fluent Design の Acrylic 効果のようなきわめてプレミアムで一体感のある極上の没入体験を低負荷で実現できる。
- Consequence: 詳細画面以外の画面（一覧画面や設定画面など）に遷移した際には、サイドバーやタイトルバーは従来のソリッドな背景色に戻り、背面背景画像も元のマイルドなぼかし（blur-2xl）と半透明（opacity-50）の通常仕様へとスムーズに復元される。
- Links: [[architecture-map]], [[quality-gates]]

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

## 2026-06-03: リリースタグは維持し、タイトルだけバージョン表示に寄せる

- Context: 既存 GitHub Release の asset には download_count があり、release や asset を削除・再作成すると履歴が失われる。
- Decision: 既存の `YYYYMMDD` タグと asset は維持し、Release title だけ `0.1.0` から `0.6.1` までの人工的な semver 形式へ更新する。大きい差分のリリースだけ minor を進め、小さい更新は patch として扱う。今後の release workflow は `release-display-version.txt` の値を title として使う。
- Rationale: タグや asset URL の互換性を守りながら、GitHub の Release 一覧で日付タグがそのままタイトルに見える状態を避けられる。
- Consequence: タグ URL と download URL は従来どおり日付タグを含む。完全な semver タグ移行は別判断として扱う。
- Links: [[maintenance-routine]], [[quality-gates]]
