<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type {
    LocalDataStream,
    LocalStream,
    RoomPublication,
  } from "@skyway-sdk/room";
  import type {
    InitMessage,
    LibraryRequestMessage,
    LibraryResponseMessage,
    MemoMessage,
    ControlStatusMessage,
    ControlStatusRequestMessage,
    PauseToggleMessage,
    PingMessage,
    RemoteGameSummary,
    ScreenshotResultMessage,
    TakeScreenshotMessage,
  } from "@/store/skywayMessage";
  import { SKYWAY_CONNECT_ENDPOINT } from "@/lib/mobileCompanionUrl";

  type ConnectionState =
    | "idle"
    | "connecting"
    | "connected"
    | "offline"
    | "error"
    | "missing";
  type ViewMode = "home" | "library" | "detail" | "controller";
  type LibraryFilter =
    | "all"
    | "playing"
    | "unplayed"
    | "cleared"
    | "liked"
    | "installed";

  type MobileOutgoingMessage =
    | PingMessage
    | InitMessage
    | MemoMessage
    | TakeScreenshotMessage
    | LibraryRequestMessage
    | ControlStatusRequestMessage
    | PauseToggleMessage;

  type PcMessage =
    | PingMessage
    | MemoMessage
    | LibraryResponseMessage
    | ScreenshotResultMessage
    | ControlStatusMessage
    | {
        type: "init_response";
        gameId: number;
        initialMemo: MemoMessage;
      };

  type LibraryCache = {
    games: RemoteGameSummary[];
    syncedAt: string;
  };

  const LIBRARY_CACHE_KEY = "launcherg-mobile-companion-library-v1";
  const PLAY_STATUS = {
    Unplayed: 0,
    Playing: 1,
    Cleared: 2,
    Interrupted: 3,
    LegacyShelved: 4,
  } as const;

  const statusLabels: Record<number, string> = {
    [PLAY_STATUS.Unplayed]: "未プレイ",
    [PLAY_STATUS.Playing]: "プレイ中",
    [PLAY_STATUS.Cleared]: "クリア済み",
    [PLAY_STATUS.Interrupted]: "中断",
    [PLAY_STATUS.LegacyShelved]: "中断",
  };

  const filterLabels: Record<LibraryFilter, string> = {
    all: "すべて",
    playing: "プレイ中",
    unplayed: "未プレイ",
    cleared: "クリア済み",
    liked: "お気に入り",
    installed: "導入済み",
  };

  const libraryFilters: LibraryFilter[] = [
    "all",
    "playing",
    "unplayed",
    "cleared",
    "liked",
    "installed",
  ];

  let connectionState: ConnectionState = "idle";
  let activeView: ViewMode = "home";
  let statusText = "接続待ち";
  let connectionErrorText = "";
  let dataStream: LocalDataStream | undefined;
  let cleanupCallbacks: (() => void)[] = [];
  let games: RemoteGameSummary[] = [];
  let selectedGameId: number | null = null;
  let memoText = "";
  let searchText = "";
  let libraryFilter: LibraryFilter = "all";
  let isMemoOpen = false;
  let isSendingScreenshot = false;
  let isPaused = false;
  let isTogglingPause = false;
  let lastActionText = "";
  let cachedAt: string | null = null;
  let didReceiveLibrary = false;
  let libraryRequestAttempts = 0;
  let libraryRetryTimer: ReturnType<typeof setInterval> | undefined;
  let controlStatusTimer: ReturnType<typeof setInterval> | undefined;

  const companionQuery = () => {
    const params = new URLSearchParams(window.location.search);
    const queryIndex = window.location.hash.indexOf("?");
    if (queryIndex === -1) {
      return params;
    }

    const hashParams = new URLSearchParams(
      window.location.hash.slice(queryIndex + 1),
    );
    hashParams.forEach((value, key) => {
      params.set(key, value);
    });
    return params;
  };

  const query = companionQuery();
  const roomId = query.get("roomId") ?? "";
  const qrAuthToken = query.get("authToken") ?? "";
  const initialGameId = Number(query.get("gameId") ?? "");
  const hasInitialGameId = Number.isFinite(initialGameId) && initialGameId > 0;
  const hasRequiredParams = !!roomId;

  $: selectedGame =
    selectedGameId === null
      ? null
      : games.find((game) => game.id === selectedGameId) ?? null;
  $: playingGames = games.filter(
    (game) => game.playStatus === PLAY_STATUS.Playing,
  );
  $: recentGames = games.filter((game) => !!game.lastPlayAt).slice(0, 6);
  $: favoriteGames = games.filter((game) => game.liked);
  $: unplayedGames = games.filter(
    (game) => game.playStatus === PLAY_STATUS.Unplayed,
  );
  $: clearedGames = games.filter(
    (game) => game.playStatus === PLAY_STATUS.Cleared,
  );
  $: homePrimaryGames =
    playingGames.length > 0 ? playingGames.slice(0, 3) : recentGames.slice(0, 3);
  $: homeEmptyText =
    connectionState === "missing"
      ? "PCのスマホ連携QRから開いてください"
      : connectionState === "error"
        ? connectionErrorText || "接続に失敗しました"
        : connectionState === "connecting" || connectionState === "connected"
          ? "PCのライブラリを読み込んでいます"
          : "PCのライブラリを同期すると表示されます";
  $: libraryEmptyText =
    connectionState === "missing"
      ? "PCのスマホ連携QRから開いてください"
      : connectionState === "error"
        ? connectionErrorText || "接続に失敗しました"
        : connectionState === "connecting" || connectionState === "connected"
          ? "PCのライブラリを読み込んでいます"
          : "同期済みのゲームがありません";
  $: normalizedSearch = searchText.trim().toLowerCase();
  $: filteredGames = games.filter((game) => {
    const matchesSearch =
      !normalizedSearch ||
      `${game.title} ${game.brandName}`.toLowerCase().includes(normalizedSearch);
    if (!matchesSearch) return false;

    switch (libraryFilter) {
      case "playing":
        return game.playStatus === PLAY_STATUS.Playing;
      case "unplayed":
        return game.playStatus === PLAY_STATUS.Unplayed;
      case "cleared":
        return game.playStatus === PLAY_STATUS.Cleared;
      case "liked":
        return game.liked;
      case "installed":
        return game.installed;
      default:
        return true;
    }
  });

  const isObject = (value: unknown): value is Record<string, unknown> =>
    !!value && typeof value === "object";

  const isRemoteGameSummary = (
    value: unknown,
  ): value is RemoteGameSummary => {
    if (!isObject(value)) return false;
    return (
      typeof value.id === "number" &&
      typeof value.title === "string" &&
      typeof value.brandName === "string" &&
      typeof value.playStatus === "number" &&
      typeof value.totalPlayTimeSeconds === "number" &&
      (typeof value.lastPlayAt === "string" || value.lastPlayAt === null) &&
      typeof value.installed === "boolean" &&
      typeof value.liked === "boolean"
    );
  };

  const isPcMessage = (value: unknown): value is PcMessage => {
    if (!isObject(value) || typeof value.type !== "string") return false;

    switch (value.type) {
      case "ping":
        return true;
      case "memo":
        return typeof value.gameId === "number" && typeof value.text === "string";
      case "init_response":
        return (
          typeof value.gameId === "number" &&
          isObject(value.initialMemo) &&
          value.initialMemo.type === "memo" &&
          typeof value.initialMemo.gameId === "number" &&
          typeof value.initialMemo.text === "string"
        );
      case "library_response":
        return (
          Array.isArray(value.games) &&
          value.games.every((game) => isRemoteGameSummary(game))
        );
      case "screenshot_result":
        return (
          typeof value.gameId === "number" &&
          typeof value.ok === "boolean" &&
          (typeof value.imagePath === "string" ||
            value.imagePath === undefined) &&
          (typeof value.error === "string" || value.error === undefined)
        );
      case "control_status":
        return (
          typeof value.isPaused === "boolean" &&
          (typeof value.error === "string" || value.error === undefined)
        );
      default:
        return false;
    }
  };

  const parsePcMessage = (data: string): PcMessage | null => {
    try {
      const parsed = JSON.parse(data) as unknown;
      return isPcMessage(parsed) ? parsed : null;
    } catch {
      return null;
    }
  };

  const sortGames = (nextGames: RemoteGameSummary[]) =>
    [...nextGames].sort((a, b) => {
      const aTime = a.lastPlayAt ? new Date(a.lastPlayAt).getTime() : 0;
      const bTime = b.lastPlayAt ? new Date(b.lastPlayAt).getTime() : 0;
      return bTime - aTime || a.title.localeCompare(b.title, "ja");
    });

  const saveLibraryCache = (nextGames: RemoteGameSummary[], syncedAt: string) => {
    try {
      const cache: LibraryCache = { games: nextGames, syncedAt };
      localStorage.setItem(LIBRARY_CACHE_KEY, JSON.stringify(cache));
    } catch (error) {
      console.warn("Failed to cache mobile companion library", error);
    }
  };

  const restoreLibraryCache = () => {
    try {
      const raw = localStorage.getItem(LIBRARY_CACHE_KEY);
      if (!raw) return false;
      const parsed = JSON.parse(raw) as Partial<LibraryCache>;
      if (
        !Array.isArray(parsed.games) ||
        !parsed.games.every((game) => isRemoteGameSummary(game)) ||
        typeof parsed.syncedAt !== "string"
      ) {
        return false;
      }

      games = sortGames(parsed.games);
      cachedAt = parsed.syncedAt;
      if (selectedGameId === null && games[0]) {
        selectedGameId = games[0].id;
      }
      return true;
    } catch (error) {
      console.warn("Failed to restore mobile companion library", error);
      return false;
    }
  };

  const sendMessage = (message: MobileOutgoingMessage) => {
    if (!dataStream) return;
    dataStream.write(JSON.stringify(message));
  };

  const requestControlStatus = () => {
    sendMessage({ type: "control_status_request" });
  };

  const startControlStatusPolling = () => {
    requestControlStatus();
    if (controlStatusTimer) return;
    controlStatusTimer = setInterval(requestControlStatus, 5000);
    cleanupCallbacks.push(() => {
      if (controlStatusTimer) {
        clearInterval(controlStatusTimer);
        controlStatusTimer = undefined;
      }
    });
  };

  const requestLibrary = () => {
    if (didReceiveLibrary || libraryRequestAttempts >= 5) return;
    libraryRequestAttempts += 1;
    sendMessage({ type: "library_request" });
    if (hasInitialGameId) {
      sendMessage({ type: "init", gameId: initialGameId });
    }
  };

  const startLibrarySync = () => {
    if (libraryRetryTimer) return;
    requestLibrary();
    libraryRetryTimer = setInterval(() => {
      if (didReceiveLibrary || libraryRequestAttempts >= 5) {
        if (libraryRetryTimer) {
          clearInterval(libraryRetryTimer);
          libraryRetryTimer = undefined;
        }
        return;
      }
      requestLibrary();
    }, 1200);
    cleanupCallbacks.push(() => {
      if (libraryRetryTimer) {
        clearInterval(libraryRetryTimer);
        libraryRetryTimer = undefined;
      }
    });
  };

  const selectGame = (game: RemoteGameSummary, nextView: ViewMode = "detail") => {
    selectedGameId = game.id;
    memoText = "";
    isMemoOpen = false;
    lastActionText = `${game.title}を選択しました`;
    activeView = nextView;
    sendMessage({ type: "init", gameId: game.id });
    requestControlStatus();
  };

  const openFilteredLibrary = (filter: LibraryFilter) => {
    libraryFilter = filter;
    activeView = "library";
  };

  const formatPlayTime = (seconds: number) => {
    if (seconds <= 0) return "0分";
    if (seconds < 7200) return `${Math.floor(seconds / 60)}分`;
    return `${Math.floor((seconds / 3600) * 10) / 10}時間`;
  };

  const formatLastPlay = (value: string | null) => {
    if (!value) return "未プレイ";
    return new Date(value).toLocaleDateString("ja-JP", {
      month: "2-digit",
      day: "2-digit",
    });
  };

  const formatSyncTime = (value: string | null) => {
    if (!value) return "未同期";
    return new Date(value).toLocaleTimeString("ja-JP", {
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const applyLibrary = (nextGames: RemoteGameSummary[]) => {
    const syncedAt = new Date().toISOString();
    didReceiveLibrary = true;
    if (libraryRetryTimer) {
      clearInterval(libraryRetryTimer);
      libraryRetryTimer = undefined;
    }

    games = sortGames(nextGames);
    cachedAt = syncedAt;
    saveLibraryCache(games, syncedAt);

    if (selectedGameId === null) {
      const initialGame = hasInitialGameId
        ? games.find((game) => game.id === initialGameId)
        : undefined;
      const currentPlayingGame = games.find(
        (game) => game.playStatus === PLAY_STATUS.Playing,
      );
      selectedGameId =
        (initialGame ?? currentPlayingGame ?? games[0] ?? null)?.id ?? null;
    }

    statusText = `${games.length}本 同期済み`;
    connectionState = "connected";
  };

  const handlePcMessage = (message: PcMessage) => {
    if (message.type === "library_response") {
      applyLibrary(message.games);
      return;
    }

    if (message.type === "init_response") {
      if (selectedGameId === null) {
        selectedGameId = message.gameId;
      }
      if (message.gameId === selectedGameId) {
        memoText = message.initialMemo.text;
      }
      return;
    }

    if (message.type === "control_status") {
      isPaused = message.isPaused;
      isTogglingPause = false;
      if (message.error) {
        lastActionText = message.error;
      }
      return;
    }

    if (message.type === "screenshot_result") {
      if (message.gameId !== selectedGameId) return;
      isSendingScreenshot = false;
      lastActionText = message.ok
        ? "スクリーンショットを保存しました"
        : message.error || "スクリーンショットの保存に失敗しました";
      return;
    }

    if (message.type === "memo" && message.gameId === selectedGameId) {
      memoText = message.text;
    }
  };

  const getSkyWayAuthToken = async () => {
    if (qrAuthToken) {
      return qrAuthToken;
    }

    const response = await fetch(SKYWAY_CONNECT_ENDPOINT, {
      method: "POST",
    });
    const { authToken } = (await response.json()) as { authToken: string };
    return authToken;
  };

  const assertCanUseSkyWay = () => {
    if (window.isSecureContext) {
      return;
    }

    throw new Error(
      "スマホからPCのdev serverをHTTPで開いているため接続できません。HTTPSのPWAで開いてください。",
    );
  };

  const connect = async () => {
    if (!hasRequiredParams) {
      connectionState = games.length > 0 ? "offline" : "missing";
      statusText = games.length > 0 ? "前回同期" : "PC未接続";
      return;
    }

    connectionState = "connecting";
    statusText = "PCに接続中";

    assertCanUseSkyWay();
    const { SkyWayContext, SkyWayRoom, SkyWayStreamFactory } = await import(
      "@skyway-sdk/room"
    );
    const authToken = await getSkyWayAuthToken();
    const context = await SkyWayContext.Create(authToken);
    const room = await SkyWayRoom.FindOrCreate(context, {
      type: "p2p",
      name: roomId,
    });
    const me = await room.join();

    dataStream = await SkyWayStreamFactory.createDataStream();
    await me.publish(dataStream);

    const subscribePcStream = async (
      publication: RoomPublication<LocalStream>,
    ) => {
      if (publication.publisher.id === me.id) return;
      if (publication.contentType !== "data") return;

      const { stream } = await me.subscribe(publication.id);
      if (stream.contentType !== "data") return;

      const { removeListener } = stream.onData.add((data) => {
        if (typeof data !== "string") return;
        const message = parsePcMessage(data);
        if (!message) return;
        handlePcMessage(message);
      });
      cleanupCallbacks.push(removeListener);

      connectionState = "connected";
      statusText = "ライブラリ同期中";
      startLibrarySync();
      startControlStatusPolling();
    };

    room.publications.forEach((publication) => {
      void subscribePcStream(publication);
    });
    room.onStreamPublished.add((event) => {
      void subscribePcStream(event.publication);
    });

    const pingTimer = setInterval(() => {
      sendMessage({ type: "ping" });
    }, 10000);
    cleanupCallbacks.push(() => clearInterval(pingTimer));
  };

  const refreshLibrary = () => {
    didReceiveLibrary = false;
    libraryRequestAttempts = 0;
    startLibrarySync();
  };

  const takeScreenshot = () => {
    if (!dataStream || selectedGameId === null) return;
    isSendingScreenshot = true;
    sendMessage({
      type: "take_screenshot",
      gameId: selectedGameId,
      cursorLine: Math.max(0, memoText.split("\n").length - 1),
    });
    lastActionText = "スクリーンショットをPCに要求しました";
    setTimeout(() => {
      isSendingScreenshot = false;
    }, 12000);
  };

  const togglePause = () => {
    if (!dataStream) return;
    isTogglingPause = true;
    sendMessage({ type: "pause_toggle" });
    lastActionText = isPaused ? "Pause解除をPCに要求しました" : "PauseをPCに要求しました";
    setTimeout(() => {
      isTogglingPause = false;
    }, 5000);
  };

  const syncMemo = () => {
    if (selectedGameId === null) return;
    sendMessage({ type: "memo", gameId: selectedGameId, text: memoText });
    lastActionText = "メモを同期しました";
  };

  onMount(() => {
    const restored = restoreLibraryCache();
    if (restored && !hasRequiredParams) {
      connectionState = "offline";
      statusText = "前回同期";
    }

    void connect().catch((error) => {
      console.error("Mobile companion connection failed", error);
      connectionErrorText =
        error instanceof Error && error.message
          ? error.message
          : "接続に失敗しました";
      connectionState = games.length > 0 ? "offline" : "error";
      statusText = games.length > 0 ? "PC未接続" : "接続に失敗しました";
    });
  });

  onDestroy(() => {
    cleanupCallbacks.forEach((callback) => callback());
    cleanupCallbacks = [];
  });
</script>

<svelte:head>
  <title>Launcherg スマホ連携</title>
</svelte:head>

<main class="mobile-shell">
  <header class="topbar">
    <div class="min-w-0">
      <div class="eyebrow">Launcherg</div>
      <h1>スマホ連携</h1>
    </div>
    <div
      class:connected={connectionState === "connected"}
      class:offline={connectionState === "offline"}
      class:error={connectionState === "error" || connectionState === "missing"}
      class="status-pill"
    >
      {statusText}
    </div>
  </header>

  <section class="content">
    {#if activeView === "home"}
      <section class="hero-strip">
        <div>
          <div class="eyebrow">同期</div>
          <div class="hero-title">{games.length}本</div>
        </div>
        <div>
          <div class="eyebrow">最終</div>
          <div class="hero-title">{formatSyncTime(cachedAt)}</div>
        </div>
        <div>
          <div class="eyebrow">Pause</div>
          <div class:paused={isPaused} class="hero-title pause-state">
            {isPaused ? "中断中" : "記録中"}
          </div>
        </div>
        <button
          type="button"
          class="icon-action"
          disabled={connectionState !== "connected"}
          on:click={refreshLibrary}
          aria-label="更新"
        >
          <span class="i-material-symbols:sync-rounded text-[22px]" />
        </button>
      </section>

      {#if homePrimaryGames.length > 0}
        <section class="section">
          <div class="section-head">
            <h2>{playingGames.length > 0 ? "現在プレイ中" : "最近"}</h2>
            <button type="button" on:click={() => (activeView = "library")}>
              すべて
            </button>
          </div>
          <div class="list">
            {#each homePrimaryGames as game (game.id)}
              <button
                type="button"
                class="game-row prominent"
                on:click={() => selectGame(game)}
              >
                <div class="min-w-0 flex-1 text-left">
                  <div class="game-title">{game.title}</div>
                  <div class="game-meta">
                    {statusLabels[game.playStatus] ?? "不明"} / {formatPlayTime(game.totalPlayTimeSeconds)}
                  </div>
                </div>
                <span class="i-material-symbols:chevron-right-rounded text-[22px] text-white/45" />
              </button>
            {/each}
          </div>
        </section>
      {:else}
        <section class="empty-state">{homeEmptyText}</section>
      {/if}

      {#if games.length > 0}
        <section class="stats-grid">
          <button type="button" on:click={() => openFilteredLibrary("unplayed")}>
            <span>{unplayedGames.length}</span>
            <small>未プレイ</small>
          </button>
          <button type="button" on:click={() => openFilteredLibrary("cleared")}>
            <span>{clearedGames.length}</span>
            <small>クリア</small>
          </button>
          <button type="button" on:click={() => openFilteredLibrary("liked")}>
            <span>{favoriteGames.length}</span>
            <small>お気に入り</small>
          </button>
        </section>
      {/if}
    {:else if activeView === "library"}
      <section class="section">
        <div class="library-head">
          <div>
            <h2>ゲーム一覧</h2>
            <div class="subtle">{filteredGames.length} / {games.length} 本</div>
          </div>
          <input
            bind:value={searchText}
            type="search"
            placeholder="検索"
            class="search-input"
          />
        </div>

        <div class="filter-strip" aria-label="絞り込み">
          {#each libraryFilters as filter}
            <button
              type="button"
              class:active={libraryFilter === filter}
              on:click={() => (libraryFilter = filter)}
            >
              {filterLabels[filter]}
            </button>
          {/each}
        </div>

        <div class="list">
          {#if games.length === 0}
            <div class="empty-state">{libraryEmptyText}</div>
          {:else if filteredGames.length === 0}
            <div class="empty-state">該当するゲームがありません</div>
          {:else}
            {#each filteredGames as game (game.id)}
              <button
                type="button"
                class:selected={game.id === selectedGameId}
                class="game-row"
                on:click={() => selectGame(game)}
              >
                <div class="min-w-0 flex-1 text-left">
                  <div class="game-title">{game.title}</div>
                  <div class="game-meta">{game.brandName || "ブランド未設定"}</div>
                </div>
                <div class="row-stat">
                  <strong>{statusLabels[game.playStatus] ?? "不明"}</strong>
                  <span>{formatLastPlay(game.lastPlayAt)}</span>
                </div>
              </button>
            {/each}
          {/if}
        </div>
      </section>
    {:else if activeView === "detail"}
      {#if selectedGame}
        <section class="detail-panel">
          <button type="button" class="back-button" on:click={() => (activeView = "library")}>
            <span class="i-material-symbols:arrow-back-rounded text-[20px]" />
            <span>一覧</span>
          </button>
          <div class="detail-title">{selectedGame.title}</div>
          <div class="detail-brand">{selectedGame.brandName || "ブランド未設定"}</div>

          <div class="detail-stats">
            <div>
              <span>{statusLabels[selectedGame.playStatus] ?? "不明"}</span>
              <small>状態</small>
            </div>
            <div>
              <span>{formatPlayTime(selectedGame.totalPlayTimeSeconds)}</span>
              <small>時間</small>
            </div>
            <div>
              <span>{formatLastPlay(selectedGame.lastPlayAt)}</span>
              <small>最終</small>
            </div>
          </div>

          <div class="action-grid">
            <button
              type="button"
              class="primary-action"
              disabled={connectionState !== "connected"}
              on:click={() => (activeView = "controller")}
            >
              <span class="i-material-symbols:gamepad-outline-rounded text-[24px]" />
              <span>操作</span>
            </button>
            <button
              type="button"
              class="secondary-action"
              disabled={connectionState !== "connected" || isSendingScreenshot}
              on:click={takeScreenshot}
            >
              <span class="i-material-symbols:photo-camera-outline-rounded text-[22px]" />
              <span>スクショ</span>
            </button>
            <button
              type="button"
              class:paused-action={isPaused}
              class="secondary-action"
              disabled={connectionState !== "connected" || isTogglingPause}
              on:click={togglePause}
            >
              <span
                class={`${isPaused
                  ? "i-material-symbols:play-arrow-rounded"
                  : "i-material-symbols:pause-rounded"} text-[22px]`}
              />
              <span>{isPaused ? "再開" : "Pause"}</span>
            </button>
          </div>

          <button
            type="button"
            class="memo-toggle"
            disabled={connectionState !== "connected"}
            on:click={() => (isMemoOpen = !isMemoOpen)}
          >
            <span class="i-material-symbols:edit-note-outline-rounded text-[22px]" />
            <span>メモ</span>
          </button>

          {#if isMemoOpen}
            <section class="memo-panel">
              <textarea bind:value={memoText} />
              <button
                type="button"
                disabled={connectionState !== "connected"}
                on:click={syncMemo}
              >
                同期
              </button>
            </section>
          {/if}
        </section>
      {:else}
        <section class="empty-state">ゲームを選択してください</section>
      {/if}
    {:else if activeView === "controller"}
      {#if selectedGame}
        <section class="controller-panel">
          <button type="button" class="back-button" on:click={() => (activeView = "detail")}>
            <span class="i-material-symbols:arrow-back-rounded text-[20px]" />
            <span>詳細</span>
          </button>
          <div class="controller-game">{selectedGame.title}</div>
          <div class="controller-status">
            {connectionState === "connected" ? "PC接続中" : "PC未接続"} / {isPaused ? "Pause中" : "記録中"} / {formatPlayTime(selectedGame.totalPlayTimeSeconds)}
          </div>

          <button
            type="button"
            class:paused-action={isPaused}
            class="pause-button"
            disabled={connectionState !== "connected" || isTogglingPause}
            on:click={togglePause}
          >
            <span
              class={`${isPaused
                ? "i-material-symbols:play-arrow-rounded"
                : "i-material-symbols:pause-rounded"} text-[28px]`}
            />
            <span>{isPaused ? "記録再開" : "Pause"}</span>
          </button>

          <button
            type="button"
            class="shutter-button"
            disabled={connectionState !== "connected" || isSendingScreenshot}
            on:click={takeScreenshot}
          >
            <span class="i-material-symbols:photo-camera-outline-rounded text-[34px]" />
            <span>スクショ</span>
          </button>

          <button
            type="button"
            class="controller-memo"
            disabled={connectionState !== "connected"}
            on:click={() => (isMemoOpen = !isMemoOpen)}
          >
            <span class="i-material-symbols:edit-note-outline-rounded text-[24px]" />
            <span>メモ</span>
          </button>

          {#if isMemoOpen}
            <section class="memo-panel">
              <textarea bind:value={memoText} />
              <button
                type="button"
                disabled={connectionState !== "connected"}
                on:click={syncMemo}
              >
                同期
              </button>
            </section>
          {/if}
        </section>
      {:else}
        <section class="empty-state">ゲームを選択してください</section>
      {/if}
    {/if}

    {#if lastActionText}
      <div class="toast-line">{lastActionText}</div>
    {/if}
  </section>

  <nav class="bottom-nav" aria-label="スマホ連携">
    <button
      type="button"
      class:active={activeView === "home"}
      on:click={() => (activeView = "home")}
    >
      <span class="i-material-symbols:home-outline-rounded text-[22px]" />
      <span>ホーム</span>
    </button>
    <button
      type="button"
      class:active={activeView === "library"}
      on:click={() => (activeView = "library")}
    >
      <span class="i-material-symbols:format-list-bulleted-rounded text-[22px]" />
      <span>一覧</span>
    </button>
    <button
      type="button"
      class:active={activeView === "controller"}
      disabled={!selectedGame}
      on:click={() => (activeView = "controller")}
    >
      <span class="i-material-symbols:gamepad-outline-rounded text-[22px]" />
      <span>操作</span>
    </button>
  </nav>
</main>

<style>
  .mobile-shell {
    min-height: 100vh;
    background: #151515;
    color: white;
    display: flex;
    flex-direction: column;
    max-width: 620px;
    margin: 0 auto;
  }

  .topbar {
    min-height: 64px;
    padding: 14px 16px 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid rgb(255 255 255 / 0.1);
    background: #191919;
  }

  .eyebrow,
  .subtle {
    font-size: 12px;
    font-weight: 600;
    color: rgb(255 255 255 / 0.52);
  }

  h1,
  h2 {
    margin: 0;
    letter-spacing: 0;
  }

  h1 {
    margin-top: 2px;
    font-size: 22px;
    line-height: 1.15;
  }

  h2 {
    font-size: 18px;
  }

  .status-pill {
    max-width: 158px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    border: 1px solid rgb(255 255 255 / 0.14);
    border-radius: 999px;
    padding: 7px 10px;
    color: rgb(255 255 255 / 0.72);
    background: rgb(255 255 255 / 0.06);
    font-size: 12px;
    font-weight: 700;
  }

  .status-pill.connected {
    border-color: rgb(94 201 142 / 0.55);
    color: #9ae6b4;
  }

  .status-pill.offline {
    border-color: rgb(245 158 11 / 0.5);
    color: #fbd38d;
  }

  .status-pill.error {
    border-color: rgb(252 129 129 / 0.45);
    color: #feb2b2;
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 14px 14px 92px;
    display: grid;
    align-content: start;
    gap: 14px;
  }

  .hero-strip,
  .section,
  .detail-panel,
  .controller-panel,
  .empty-state,
  .toast-line {
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(255 255 255 / 0.045);
  }

  .hero-strip {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr)) auto;
    align-items: center;
    gap: 12px;
    padding: 12px;
  }

  .hero-title {
    margin-top: 3px;
    font-size: 20px;
    font-weight: 800;
  }

  .pause-state {
    color: #9ae6b4;
  }

  .pause-state.paused {
    color: #fbd38d;
  }

  .icon-action {
    height: 48px;
    width: 48px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    border: 1px solid rgb(255 255 255 / 0.12);
    background: rgb(255 255 255 / 0.07);
    color: white;
  }

  .section,
  .detail-panel,
  .controller-panel {
    padding: 14px;
  }

  .section-head,
  .library-head {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 12px;
  }

  .section-head button,
  .back-button {
    border: 0;
    background: transparent;
    color: #9ae6b4;
    font-size: 13px;
    font-weight: 800;
  }

  .back-button {
    height: 36px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 0;
  }

  .list {
    display: grid;
    gap: 8px;
    margin-top: 12px;
  }

  .game-row {
    min-height: 68px;
    display: flex;
    align-items: center;
    gap: 12px;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(255 255 255 / 0.045);
    color: white;
    padding: 10px 12px;
  }

  .game-row.prominent {
    min-height: 76px;
  }

  .game-row.selected {
    border-color: rgb(94 201 142 / 0.55);
    background: rgb(94 201 142 / 0.13);
  }

  .game-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
    font-weight: 800;
  }

  .game-meta {
    margin-top: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: rgb(255 255 255 / 0.5);
    font-size: 12px;
    font-weight: 600;
  }

  .row-stat {
    flex: 0 0 auto;
    min-width: 74px;
    text-align: right;
  }

  .row-stat strong,
  .row-stat span {
    display: block;
  }

  .row-stat strong {
    color: rgb(255 255 255 / 0.78);
    font-size: 12px;
  }

  .row-stat span {
    margin-top: 4px;
    color: rgb(255 255 255 / 0.42);
    font-size: 11px;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
  }

  .stats-grid button {
    min-height: 74px;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(255 255 255 / 0.045);
    color: white;
  }

  .stats-grid span,
  .stats-grid small {
    display: block;
  }

  .stats-grid span {
    font-size: 22px;
    font-weight: 900;
  }

  .stats-grid small {
    margin-top: 3px;
    color: rgb(255 255 255 / 0.48);
    font-size: 11px;
    font-weight: 700;
  }

  .search-input {
    height: 42px;
    min-width: 0;
    flex: 1;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(0 0 0 / 0.22);
    color: white;
    padding: 0 12px;
    outline: none;
    font-size: 14px;
  }

  .search-input::placeholder {
    color: rgb(255 255 255 / 0.35);
  }

  .filter-strip {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding: 12px 0 2px;
  }

  .filter-strip button {
    min-height: 36px;
    white-space: nowrap;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 999px;
    background: rgb(255 255 255 / 0.05);
    color: rgb(255 255 255 / 0.68);
    padding: 0 12px;
    font-size: 13px;
    font-weight: 800;
  }

  .filter-strip button.active {
    border-color: rgb(94 201 142 / 0.55);
    background: rgb(94 201 142 / 0.16);
    color: #b7f3cb;
  }

  .detail-title,
  .controller-game {
    margin-top: 10px;
    font-size: 21px;
    line-height: 1.25;
    font-weight: 900;
  }

  .detail-brand,
  .controller-status {
    margin-top: 6px;
    color: rgb(255 255 255 / 0.5);
    font-size: 13px;
    font-weight: 700;
  }

  .detail-stats {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
    margin-top: 16px;
  }

  .detail-stats div {
    min-height: 66px;
    border-radius: 8px;
    background: rgb(255 255 255 / 0.055);
    padding: 10px;
  }

  .detail-stats span,
  .detail-stats small {
    display: block;
  }

  .detail-stats span {
    font-size: 14px;
    font-weight: 900;
  }

  .detail-stats small {
    margin-top: 7px;
    color: rgb(255 255 255 / 0.45);
    font-size: 11px;
    font-weight: 700;
  }

  .action-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-top: 16px;
  }

  .primary-action,
  .secondary-action,
  .memo-toggle,
  .controller-memo {
    min-height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 1px solid rgb(255 255 255 / 0.12);
    border-radius: 8px;
    font-size: 14px;
    font-weight: 900;
  }

  .primary-action {
    background: #5ec98e;
    color: #101410;
  }

  .secondary-action,
  .memo-toggle,
  .controller-memo {
    background: rgb(255 255 255 / 0.06);
    color: rgb(255 255 255 / 0.9);
  }

  .secondary-action.paused-action,
  .pause-button.paused-action {
    border-color: rgb(245 158 11 / 0.5);
    background: rgb(245 158 11 / 0.18);
    color: #fbd38d;
  }

  .memo-toggle,
  .controller-memo {
    width: 100%;
    margin-top: 10px;
  }

  .memo-panel {
    margin-top: 10px;
    display: grid;
    gap: 10px;
  }

  .memo-panel textarea {
    min-height: 180px;
    resize: vertical;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(0 0 0 / 0.22);
    color: white;
    padding: 12px;
    outline: none;
    font-size: 15px;
    line-height: 1.6;
  }

  .memo-panel button {
    min-height: 46px;
    border: 0;
    border-radius: 8px;
    background: white;
    color: #151515;
    font-weight: 900;
  }

  .controller-panel {
    min-height: calc(100vh - 190px);
    display: flex;
    flex-direction: column;
  }

  .pause-button {
    min-height: 64px;
    width: 100%;
    margin-top: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    border: 1px solid rgb(255 255 255 / 0.12);
    border-radius: 8px;
    background: rgb(255 255 255 / 0.06);
    color: rgb(255 255 255 / 0.9);
    font-size: 18px;
    font-weight: 950;
  }

  .shutter-button {
    min-height: 152px;
    width: 100%;
    margin-top: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    border: 0;
    border-radius: 8px;
    background: #5ec98e;
    color: #101410;
    font-size: 23px;
    font-weight: 950;
  }

  .empty-state {
    min-height: 132px;
    display: grid;
    place-items: center;
    color: rgb(255 255 255 / 0.52);
    font-size: 13px;
    font-weight: 700;
    text-align: center;
    padding: 16px;
  }

  .toast-line {
    padding: 10px 12px;
    color: rgb(255 255 255 / 0.72);
    font-size: 13px;
    font-weight: 700;
  }

  .bottom-nav {
    position: fixed;
    left: 50%;
    bottom: 0;
    transform: translateX(-50%);
    width: min(620px, 100vw);
    height: 72px;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    border-top: 1px solid rgb(255 255 255 / 0.1);
    background: rgb(25 25 25 / 0.96);
    backdrop-filter: blur(14px);
  }

  .bottom-nav button {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    border: 0;
    background: transparent;
    color: rgb(255 255 255 / 0.54);
    font-size: 11px;
    font-weight: 800;
  }

  .bottom-nav button.active {
    color: #9ae6b4;
  }

  button:disabled {
    pointer-events: none;
    opacity: 0.42;
  }

  button:active {
    transform: translateY(1px);
  }
</style>
