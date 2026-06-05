---
id: remote-play-companion-ux-research
title: Remote Play Companion UX Research
type: research-brief
status: active
updated: 2026-06-05
links:
  - external-research
  - idea-pipeline
  - remote-play-hub
  - mobile-companion-service-blueprint
---

# Remote Play Companion UX Research

## Question

PC画面をスマホへ移して遊ぶ時、Launcherg-Modはどの体験を自前で持ち、どの体験を既存サービスへ任せるべきか。特に、プレイ中にPC画面を切り替えずにメモ、スクリーンショット、Pauseを扱う補助コントローラーの形を決める。

## Sources

- Source: Steam Remote Play / Touch Controller
  - URL: https://partner.steamgames.com/doc/features/remoteplay?language=english
  - Checked: 2026-06-05
  - Why reliable: Steamworks公式ドキュメント。Remote Play on Phone/Tabletとタッチコントローラー設定の考え方が分かる。
- Source: Moonlight Setup Guide
  - URL: https://github.com/moonlight-stream/moonlight-docs/wiki/Setup-Guide
  - Checked: 2026-06-05
  - Why reliable: Moonlight公式ドキュメント。Sunshineとのペアリング、ショートカット、iOS要件が分かる。
- Source: Touch Portal
  - URL: https://touchportal.com/
  - Checked: 2026-06-05
  - Why reliable: モバイルをPC向けマクロコントローラーにする公式サイト。ボタン、ページ、マクロ、フィードバックの設計参考になる。
- Source: Unified Remote
  - URL: https://www.unifiedremote.com/features
  - Checked: 2026-06-05
  - Why reliable: PC操作リモコンの公式機能一覧。Wake-on-LAN、ショートカット、URI/Siri連携など周辺体験が分かる。
- Source: GameGlass
  - URL: https://support.gameglass.gg/en/articles/7231455-connect-to-gameglass-from-your-touchscreen-device
  - Checked: 2026-06-05
  - Why reliable: ゲーム用タッチスクリーン補助パネルの公式ヘルプ。PWAで始める導線が参考になる。
- Source: Apple UI Design Tips
  - URL: https://developer.apple.com/design/tips/
  - Checked: 2026-06-05
  - Why reliable: Apple公式。タッチ操作と44pt以上のヒットターゲットが確認できる。
- Source: NN/g Visibility of System Status
  - URL: https://www.nngroup.com/articles/visibility-system-status/
  - Checked: 2026-06-05
  - Why reliable: UX原則の定番資料。操作後の状態フィードバックの根拠になる。
- Source: PlayStation App
  - URL: https://www.playstation.com/playstation-app/
  - Checked: 2026-06-05
  - Why reliable: PlayStation公式。モバイル companion におけるゲームキャプチャ閲覧、共有、リモートダウンロード、Remote Play開始導線が分かる。
- Source: Xbox Mobile App
  - URL: https://www.xbox.com/en-US/apps/xbox-app-on-mobile
  - Checked: 2026-06-05
  - Why reliable: Xbox公式。ゲーム一覧、キャプチャ、チャット、実績、リモートプレイ、クラウドプレイをモバイルにまとめる考え方が分かる。
- Source: Steam Remote Downloads
  - URL: https://help.steampowered.com/en/faqs/view/1025-BD94-12FC-3409
  - Checked: 2026-06-05
  - Why reliable: Steam公式サポート。PCが起動している時にモバイルやWebからライブラリ管理する考え方が分かる。
- Source: Apple Bonjour
  - URL: https://developer.apple.com/bonjour/index.html
  - Checked: 2026-06-05
  - Why reliable: Apple公式。ローカルネットワーク上のサービス自動検出をiOS/macOSで扱う候補として確認した。
- Source: MDN PWA Installing
  - URL: https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Installing
  - Checked: 2026-06-05
  - Why reliable: MDN。PWAのホーム画面追加、インストール、オフライン挙動の基本を確認した。
- Source: MDN Making PWAs Installable
  - URL: https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Making_PWAs_installable
  - Checked: 2026-06-05
  - Why reliable: MDN。manifest、HTTPS/localhost要件、iOSで `beforeinstallprompt` が使えないことを確認した。
- Source: MDN Secure Contexts and Service Worker API
  - URL: https://developer.mozilla.org/en-US/docs/Web/Security/Defenses/Secure_Contexts
  - Checked: 2026-06-05
  - Why reliable: MDN。Service Workerや強力なWeb APIがsecure context前提で、LAN上の任意HTTPが同じ扱いにならないことを確認した。
- Source: WebKit Safari 26 Web Apps
  - URL: https://webkit.org/blog/17333/webkit-features-in-safari-26-0/
  - Checked: 2026-06-05
  - Why reliable: WebKit公式。iOS/iPadOS 26のホーム画面Web App挙動とmanifestの扱いを確認した。
- Source: Apple Support Turn Website Into App
  - URL: https://support.apple.com/en-ca/guide/iphone/iphea86e5236/ios
  - Checked: 2026-06-05
  - Why reliable: Apple公式サポート。iPhoneでSafariからホーム画面へ追加し、Open as Web Appを有効にする手順を確認した。
- Source: Apple TN3179 Local Network Privacy
  - URL: https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy
  - Checked: 2026-06-05
  - Why reliable: Apple公式Tech Note。Safari/WKWebViewからのローカルネットワーク通信の扱いを確認した。

## Findings

- [Steam] Steam Remote Playは、スマホ/タブレット向けにゲームごとのタッチコントローラー設定を作り、不要な操作を外して配置を調整する考え方を持つ。
- [Moonlight] Moonlight/Sunshineは映像配信と入力転送に集中しており、ペアリングやネットワーク設定が初回のつまずきになりやすい。
- [Touch Portal] Touch PortalやDeckboard系は、スマホを汎用マクロパッドにする。強力だが、自由度が高すぎるとプレイ中の迷いも増える。
- [Unified Remote] Unified Remoteは、Wake-on-LAN、ショートカット、URI/Siriなど、操作入口を複数持つ。
- [GameGlass] GameGlassは、ゲーム専用のタッチパネルをPWAとして始められる。App Store配布前にWebで体験を固める選択肢がある。
- [Apple] Appleは44pt以上のタップ領域を推奨している。補助コントローラーの主要操作は小さいアイコン列ではなく大きなボタンにすべき。
- [NN/g] 状態フィードバックが遅いと、ユーザーは操作が届いたか分からず連打しやすい。
- [PlayStation][Xbox] PlayStation AppとXbox Appは、プレイ中の遠隔操作だけでなく、キャプチャ閲覧、共有、ライブラリ管理、リモートプレイ開始をスマホ側の主要価値にしている。
- [Steam Downloads] Steam Remote Downloadsは、PC側クライアントが動いている時にスマホ/Webからライブラリ管理できる導線を持つ。
- [Bonjour] Bonjourはローカルネットワーク上のサービス発見に使える。ネイティブiOS化時のPC自動検出候補になる。
- [MDN PWA] PWAはホーム画面追加でネイティブ風に使えるが、インストール導線やオフライン/保存挙動はブラウザとOSに依存する。
- [MDN PWA Install] PWAとして扱うにはmanifestとHTTPS配信を前提にする。iOSでは `beforeinstallprompt` による任意のインストールプロンプト表示が使えないため、アプリ内で短い手順を見せる必要がある。
- [MDN Secure Context] Service Workerはsecure context専用。`localhost` は同一端末の開発用途では扱いやすいが、iPhoneからPCのLAN内HTTPへ接続する構成はPWAの安定配布面として扱いにくい。
- [WebKit iOS26][Apple Support] iPhoneではホーム画面追加とOpen as Web AppでWebアプリとして起動できる。manifestやアイコンは不要になりつつあるが、体験品質のためには引き続き用意する。
- [Apple TN3179] Safari/WKWebViewのローカルネットワーク通信はネイティブアプリのLocal Network権限とは扱いが異なるが、PWA側のHTTPS、CORS、証明書、認可の課題は別に残る。

## Interpretation for Launcherg-Mod

Launcherg-Modは汎用マクロデッキではなく、ノベルゲーム向けの補助コントローラーに絞るべき。画面は「大きな主操作」「静かな状態表示」「必要時だけ開くメモ」の3領域に分ける。映像配信はMoonlight/SunshineまたはSteam Linkへ任せ、Launcherg-Modは初回設定、起動、接続、撮影、記録、復帰をつなぐ。

Steam Linkのようにゲームごとのタッチ設定は有効だが、Launcherg-Modでは自由配置ではなく「作品別撮影プリセット」として扱う。Touch Portalのような自由マクロは避け、右クリック、Space、Hなど安全な少数入力だけを選べるようにする。

PC補助をしていない時のスマホ側は、PlayStation AppやXbox Appのように、ライブラリとスクリーンショットを見返す場所にする。ただしLauncherg-Modはローカルアプリなので、クラウド共有を前提にせず、同一LAN同期と端末内キャッシュを基本にする。

サービス全体の構成は [[mobile-companion-service-blueprint]] に分け、公開HTTPS PWAで体験検証し、必要が出たらBonjour検出やiOSネイティブ化へ進める。PWAからPCローカルAPIを直接叩く構成は初期MVPに入れず、既存SkyWay接続を短命セッションで安全化して使う。

## Candidate Ideas

- 狙い: プレイ中に見なくても押せる補助面を作る。
- 最初の小さな実装: 既存SkyWay接続ページを安全化し、Pause状態表示、Pause切替、通常スクショの送信中/成功/失敗表示だけを追加する。
- 検証方法: iPhoneを横向き/縦向きで持ち、PC画面を見たまま主要操作を押せるか確認する。
- 主なリスク: 現行SkyWayメッセージの信頼境界、フロント側 `startProcessMap` 依存、Pauseがグローバル操作であること、接続失敗時の復帰の悪さ。

- 狙い: PC補助中でない時も、登録ゲームとスクリーンショットをスマホで見返せるようにする。
- 最初の小さな実装: Controller MVP後に、同一LAN接続中のゲーム一覧、最近遊んだゲーム、作品詳細の読み取り専用ビューだけを表示する。Galleryはさらに後続に分ける。
- 検証方法: PCから離れた状態で、次に遊ぶ作品を選べるか、撮ったスクリーンショットを探せるかを確認する。
- 主なリスク: PCが起動していない時の期待値、画像転送量、ローカルパスや成人向け画像のプライバシー、削除操作の誤爆。

## Decision

PWAとして提供する。ただし最初はPWA全体のHome/Library/Galleryではなく、公開HTTPS PWA上の既存SkyWay画面を安全化し、Pause/通常スクショ補助に絞る。読み取り専用ライブラリ、Gallery、ネイティブiOS化は後続判断にする。
