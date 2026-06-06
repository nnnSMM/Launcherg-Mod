<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { saveImageToCache, getAllCachedImages } from "@/lib/imageCache";
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
    ImageMetadataMessage,
    PauseToggleMessage,
    PingMessage,
    RemoteGameSummary,
    ScreenshotResultMessage,
    TakeScreenshotMessage,
    ThumbnailRequestMessage,
  } from "@/store/skywayMessage";
  import { configureMobileCompanionInstallManifest } from "@/lib/mobileCompanionInstall";
  import { SKYWAY_CONNECT_ENDPOINT } from "@/lib/mobileCompanionUrl";

  type ConnectionState =
    | "idle"
    | "connecting"
    | "connected"
    | "offline"
    | "error"
    | "missing";
  type ViewMode = "library" | "detail" | "connect" | "controller";
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
    | ThumbnailRequestMessage
    | ControlStatusRequestMessage
    | PauseToggleMessage;

  type PcMessage =
    | PingMessage
    | MemoMessage
    | ImageMetadataMessage
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

  type PendingImage = {
    path: string;
    mimeType: string;
    totalChunkLength: number;
    chunks: (Uint8Array | undefined)[];
    received: number;
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
  let activeView: ViewMode = "library";
  let contentContainer: HTMLElement | undefined;
  let statusText = "接続待ち";
  let connectionErrorText = "";
  let dataStream: LocalDataStream | undefined;
  let cleanupCallbacks: (() => void)[] = [];
  let games: RemoteGameSummary[] = [];
  let selectedGameId: number | null = null;
  let memoText = "";
  let searchText = "";
  let libraryFilter: LibraryFilter = "all";
  let isSendingScreenshot = false;
  let isPaused = false;
  let isTogglingPause = false;
  let isTracking = false;
  let activeGameId: number | null = null;
  let activeProcessId: number | null = null;
  let lastActionText = "";
  let cachedAt: string | null = null;
  let didReceiveLibrary = false;
  let didSelectGameManually = false;
  let imageUrlsByPath: Record<string, string> = {};
  let pendingImages = new Map<number, PendingImage>();
  let objectUrls: string[] = [];
  let visibleThumbnailGames: RemoteGameSummary[] = [];
  let thumbnailRequestsByPath = new Map<
    string,
    { attempts: number; lastRequestedAt: number }
  >();
  let libraryRequestAttempts = 0;
  let libraryRetryTimer: ReturnType<typeof setInterval> | undefined;
  let controlStatusTimer: ReturnType<typeof setInterval> | undefined;
  let thumbnailRetryTimer: ReturnType<typeof setInterval> | undefined;

  const THUMBNAIL_BATCH_SIZE = 16;
  const THUMBNAIL_RETRY_DELAY_MS = 8000;
  const THUMBNAIL_MAX_ATTEMPTS = 3;

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

  const UI_STATE_KEY = "launcherg_mobile_ui_state";
  const restoreUiState = () => {
    try {
      const raw = localStorage.getItem(UI_STATE_KEY);
      if (raw) return JSON.parse(raw);
    } catch {
      //
    }
    return null;
  };

  const query = companionQuery();
  configureMobileCompanionInstallManifest(query);
  const roomId = query.get("roomId") ?? "";
  const qrAuthToken = query.get("authToken") ?? "";
  const requestedMode = query.get("mode");
  const initialGameId = Number(query.get("gameId") ?? "");
  const hasInitialGameId = Number.isFinite(initialGameId) && initialGameId > 0;
  const hasRequiredParams = !!roomId;

  const uiState = restoreUiState();
  if (requestedMode) {
    activeView = requestedMode === "library" ? "library" : "controller";
  } else if (uiState?.activeView && uiState.activeView !== "home") {
    activeView = uiState.activeView;
  } else {
    activeView = "library";
  }
  
  if (hasInitialGameId) {
    selectedGameId = initialGameId;
  } else if (uiState?.selectedGameId) {
    selectedGameId = uiState.selectedGameId;
  }

  if (uiState) {
    if (uiState.libraryFilter) libraryFilter = uiState.libraryFilter;
    if (uiState.searchText !== undefined) searchText = uiState.searchText;
    if (uiState.didSelectGameManually) didSelectGameManually = uiState.didSelectGameManually;
  }

  $: {
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(UI_STATE_KEY, JSON.stringify({
        activeView,
        selectedGameId,
        libraryFilter,
        searchText,
        didSelectGameManually,
      }));
    }
  }

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
  $: activeGame =
    activeGameId === null
      ? null
      : games.find((game) => game.id === activeGameId) ?? null;
  $: canControl =
    connectionState === "connected" &&
    isTracking &&
    activeGameId !== null &&
    selectedGameId === activeGameId &&
    selectedGame !== null;
  $: showNowPlayingBar =
    activeView !== "controller" &&
    activeView !== "detail" &&
    selectedGame !== null &&
    connectionState === "connected" &&
    selectedGameId === activeGameId;
  $: memoPreview = memoText.trim();
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
  $: visibleThumbnailGames =
    activeView === "library"
      ? filteredGames
      : selectedGame
        ? [selectedGame]
        : [];
  $: if (connectionState === "connected") {
    requestThumbnailsForGames(visibleThumbnailGames);
  }

  $: if (activeView === "detail" && contentContainer) {
    void tick().then(() => {
      if (contentContainer) {
        contentContainer.scrollTop = 0;
      }
    });
  }

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
      typeof value.liked === "boolean" &&
      (value.thumbnailPath === undefined ||
        typeof value.thumbnailPath === "string" ||
        value.thumbnailPath === null) &&
      (value.thumbnailWidth === undefined ||
        typeof value.thumbnailWidth === "number" ||
        value.thumbnailWidth === null) &&
      (value.thumbnailHeight === undefined ||
        typeof value.thumbnailHeight === "number" ||
        value.thumbnailHeight === null)
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
      case "image_metadata":
        return (
          typeof value.path === "string" &&
          typeof value.key === "number" &&
          typeof value.totalChunkLength === "number" &&
          typeof value.mimeType === "string"
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
          (typeof value.isTracking === "boolean" ||
            value.isTracking === undefined) &&
          (typeof value.activeGameId === "number" ||
            value.activeGameId === null ||
            value.activeGameId === undefined) &&
          (typeof value.activeProcessId === "number" ||
            value.activeProcessId === null ||
            value.activeProcessId === undefined) &&
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

  const handleImageMetadata = (message: ImageMetadataMessage) => {
    if (message.totalChunkLength <= 0) return;
    pendingImages.set(message.key, {
      path: message.path,
      mimeType: message.mimeType,
      totalChunkLength: message.totalChunkLength,
      chunks: new Array(message.totalChunkLength),
      received: 0,
    });
  };

  const toImageChunk = (data: unknown): Uint8Array | null => {
    if (data instanceof Uint8Array) return data;
    if (data instanceof ArrayBuffer) return new Uint8Array(data);
    if (ArrayBuffer.isView(data)) {
      return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    }
    return null;
  };

  const handleImageChunk = (data: unknown) => {
    const chunk = toImageChunk(data);
    if (!chunk || chunk.byteLength < 2) return;

    const key = chunk[0];
    const index = chunk[1];
    const pending = pendingImages.get(key);
    if (!pending || index >= pending.totalChunkLength) return;
    if (!pending.chunks[index]) {
      pending.received += 1;
    }
    pending.chunks[index] = chunk.slice(2);

    if (pending.received !== pending.totalChunkLength) return;
    const blob = new Blob(pending.chunks as Uint8Array[], {
      type: pending.mimeType,
    });
    const previousUrl = imageUrlsByPath[pending.path];
    if (previousUrl) {
      URL.revokeObjectURL(previousUrl);
    }
    const imageUrl = URL.createObjectURL(blob);
    objectUrls = [...objectUrls, imageUrl];
    imageUrlsByPath = { ...imageUrlsByPath, [pending.path]: imageUrl };
    pendingImages.delete(key);
    
    void saveImageToCache(pending.path, blob);
  };

  const gameThumbnailUrl = (game: RemoteGameSummary) => {
    const path = game.thumbnailPath?.trim();
    return path ? imageUrlsByPath[path] : undefined;
  };

  const sendMessage = (message: MobileOutgoingMessage) => {
    if (!dataStream) return;
    dataStream.write(JSON.stringify(message));
  };

  const requestThumbnailsForGames = (
    targetGames: RemoteGameSummary[],
    allowRetry = false,
  ) => {
    if (!dataStream || connectionState !== "connected") return;

    const now = Date.now();
    const paths: string[] = [];
    for (const game of targetGames) {
      const path = game.thumbnailPath?.trim();
      if (!path || imageUrlsByPath[path]) continue;

      const requestState = thumbnailRequestsByPath.get(path);
      if (requestState) {
        const canRetry =
          allowRetry &&
          requestState.attempts < THUMBNAIL_MAX_ATTEMPTS &&
          now - requestState.lastRequestedAt >= THUMBNAIL_RETRY_DELAY_MS;
        if (!canRetry) continue;
      }

      thumbnailRequestsByPath.set(path, {
        attempts: (requestState?.attempts ?? 0) + 1,
        lastRequestedAt: now,
      });
      paths.push(path);
    }

    for (let i = 0; i < paths.length; i += THUMBNAIL_BATCH_SIZE) {
      sendMessage({
        type: "thumbnail_request",
        paths: paths.slice(i, i + THUMBNAIL_BATCH_SIZE),
      });
    }
  };

  const requestVisibleThumbnails = (allowRetry = false) => {
    requestThumbnailsForGames(visibleThumbnailGames, allowRetry);
  };

  const requestControlStatus = () => {
    sendMessage({ type: "control_status_request" });
  };

  const adoptActiveGame = (nextActiveGameId: number | null) => {
    activeGameId = nextActiveGameId;
    if (nextActiveGameId === null) {
      return;
    }

    const target = games.find((game) => game.id === nextActiveGameId);
    if (!target) {
      return;
    }

    const shouldLoadMemo = selectedGameId !== target.id;
    selectedGameId = target.id;
    didSelectGameManually = false;
    if (activeView === "detail") {
      activeView = "controller";
    }
    if (shouldLoadMemo) {
      memoText = "";
      sendMessage({ type: "init", gameId: target.id });
    }
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

  const startThumbnailRetry = () => {
    requestVisibleThumbnails();
    if (thumbnailRetryTimer) return;
    thumbnailRetryTimer = setInterval(() => {
      requestVisibleThumbnails(true);
    }, THUMBNAIL_RETRY_DELAY_MS);
    cleanupCallbacks.push(() => {
      if (thumbnailRetryTimer) {
        clearInterval(thumbnailRetryTimer);
        thumbnailRetryTimer = undefined;
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
    didSelectGameManually = true;
    memoText = "";
    activeView = nextView;
    sendMessage({ type: "init", gameId: game.id });
    requestControlStatus();
  };

  const openController = () => {
    const target = activeGame ?? selectedGame ?? playingGames[0] ?? games[0];
    if (!target) return;

    if (selectedGameId !== target.id) {
      selectGame(target, "controller");
      return;
    }

    activeView = "controller";
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

    if (activeGameId !== null && games.some((game) => game.id === activeGameId)) {
      adoptActiveGame(activeGameId);
    } else if (selectedGameId === null) {
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
    requestControlStatus();
  };

  const handlePcMessage = (message: PcMessage) => {
    if (message.type === "library_response") {
      applyLibrary(message.games);
      return;
    }

    if (message.type === "image_metadata") {
      handleImageMetadata(message);
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
      isTracking = message.isTracking ?? false;
      activeProcessId = message.activeProcessId ?? null;
      adoptActiveGame(message.activeGameId ?? null);
      if (message.isTracking === false) {
        statusText = "PC接続中 / ゲーム未検知";
      } else if (typeof message.activeGameId === "number") {
        statusText = "PC接続中 / 操作可能";
      }
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

  const getConnectionFailureText = (error: unknown) => {
    const message = error instanceof Error ? error.message.toLowerCase() : "";
    if (message.includes("token")) {
      return "接続情報の期限が切れています。PC側のスマホ連携QRから開き直してください。";
    }
    if (message.includes("secure") || message.includes("https")) {
      return "公開PWAのQRから、安全な接続で開き直してください。";
    }
    return "PCに接続できません。PC側のスマホ連携QRから開き直してください。";
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
        if (typeof data !== "string") {
          handleImageChunk(data);
          return;
        }
        const message = parsePcMessage(data);
        if (!message) return;
        handlePcMessage(message);
      });
      cleanupCallbacks.push(removeListener);

      connectionState = "connected";
      statusText = "ライブラリ同期中";
      startLibrarySync();
      startControlStatusPolling();
      startThumbnailRetry();
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
    thumbnailRequestsByPath = new Map();
    startLibrarySync();
  };

  const takeScreenshot = (hideText = false) => {
    if (!dataStream || selectedGameId === null) return;
    isSendingScreenshot = true;
    sendMessage({
      type: "take_screenshot",
      gameId: selectedGameId,
      cursorLine: Math.max(0, memoText.split("\n").length - 1),
      hideText,
    });
    lastActionText = hideText
      ? "文字消しスクショをPCに要求しました"
      : "スクリーンショットをPCに要求しました";
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
    void getAllCachedImages().then((cached) => {
      cached.forEach(({ path, blob }) => {
        const imageUrl = URL.createObjectURL(blob);
        objectUrls = [...objectUrls, imageUrl];
        imageUrlsByPath = { ...imageUrlsByPath, [path]: imageUrl };
      });
    });

    const restored = restoreLibraryCache();
    if (restored && !hasRequiredParams) {
      connectionState = "offline";
      statusText = "前回同期";
    }

    void connect().catch((error) => {
      console.error("Mobile companion connection failed", error);
      connectionErrorText = getConnectionFailureText(error);
      connectionState = games.length > 0 ? "offline" : "error";
      statusText = games.length > 0 ? "PC未接続" : "接続に失敗しました";
    });
  });

  onDestroy(() => {
    cleanupCallbacks.forEach((callback) => callback());
    cleanupCallbacks = [];
    objectUrls.forEach((url) => URL.revokeObjectURL(url));
    objectUrls = [];
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
  <section class:controller-mode={activeView === "controller"} class="content" bind:this={contentContainer}>
    {#if activeView === "library"}
      <section class="library-view">
        <div class="library-toolbar">
          <div>
            <div class="eyebrow">Library</div>
            <h2>ゲーム一覧</h2>
          </div>
          <div class="subtle">{filteredGames.length} / {games.length} 本</div>
        </div>

        <label class="search-shell">
          <span class="i-material-symbols:search-rounded text-[20px]" />
          <input
            bind:value={searchText}
            type="search"
            placeholder="タイトル・ブランドで検索"
            class="search-input"
          />
        </label>

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

        <div class="game-card-list">
          {#if games.length === 0}
            <div class="empty-state">{libraryEmptyText}</div>
          {:else if filteredGames.length === 0}
            <div class="empty-state">該当するゲームがありません</div>
          {:else}
            {#each filteredGames as game (game.id)}
              {@const thumbnailUrl = gameThumbnailUrl(game)}
              <button
                type="button"
                class="game-card"
                on:click={() => selectGame(game)}
              >
                <div class="game-thumb">
                  {#if thumbnailUrl}
                    <img src={thumbnailUrl} alt="" loading="lazy" />
                  {:else}
                    <span class="i-material-symbols:image-outline-rounded text-[30px]" />
                  {/if}
                </div>
                <div class="game-card-body">
                  <div class="game-title">{game.title}</div>
                  <div class="game-meta">{game.brandName || "ブランド未設定"}</div>
                  <div class="meta-chips">
                    <span>{statusLabels[game.playStatus] ?? "不明"}</span>
                    <span>{formatLastPlay(game.lastPlayAt)}</span>
                    {#if game.installed}
                      <span>導入済み</span>
                    {/if}
                  </div>
                </div>
              </button>
            {/each}
          {/if}
        </div>
      </section>
    {:else if activeView === "detail"}
      {#if selectedGame}
        {@const selectedThumbnailUrl = gameThumbnailUrl(selectedGame)}
        <section class="detail-panel">
          <button type="button" class="back-button" on:click={() => (activeView = "library")}>
            <span class="i-material-symbols:arrow-back-rounded text-[20px]" />
            <span>一覧</span>
          </button>
          <div class="detail-kicker">Game Detail</div>
          <div class="detail-title">{selectedGame.title}</div>
          <div class="detail-brand">{selectedGame.brandName || "ブランド未設定"}</div>

          <div class="detail-thumb" class:has-image={!!selectedThumbnailUrl}>
            {#if selectedThumbnailUrl}
              <img src={selectedThumbnailUrl} alt="" />
            {:else}
              <span class="i-material-symbols:image-outline-rounded text-[42px]" />
            {/if}
          </div>

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

          <section class="memo-preview">
            <div class="section-head">
              <h2>メモ</h2>
              <span class="subtle">編集</span>
            </div>
            <div class="detail-memo-editor">
              <textarea
                bind:value={memoText}
                placeholder="メモを入力"
              />
              <button
                type="button"
                disabled={connectionState !== "connected" || selectedGameId === null}
                on:click={syncMemo}
              >
                メモを同期
              </button>
            </div>
          </section>
        </section>
      {:else}
        <section class="empty-state">ゲームを選択してください</section>
      {/if}
    {:else if activeView === "connect"}
      <section class="connect-view">
        <section class="section">
          <div class="section-head">
            <h2>PC接続</h2>
            <button
              type="button"
              disabled={connectionState !== "connected"}
              on:click={refreshLibrary}
            >
              更新
            </button>
          </div>
          <div
            class:connected={connectionState === "connected"}
            class:offline={connectionState === "offline"}
            class:error={connectionState === "error" || connectionState === "missing"}
            class="connect-card"
          >
            <span class="i-material-symbols:desktop-windows-outline-rounded text-[28px]" />
            <div>
              <strong>{statusText}</strong>
              <span>{cachedAt ? `最終同期 ${formatSyncTime(cachedAt)}` : "未同期"}</span>
            </div>
          </div>
        </section>

        <section class="section">
          <h2>再接続</h2>
          <div class="connect-copy">
            PC側のスマホ連携QRから開くと、このPWAを同じPCに接続できます。ホーム画面アイコンが古い場合は削除して追加し直してください。
          </div>
          <button type="button" class="secondary-full-action" on:click={() => window.location.reload()}>
            <span class="i-material-symbols:refresh-rounded text-[22px]" />
            <span>再読み込み</span>
          </button>
        </section>
      </section>
    {:else if activeView === "controller"}
      {#if canControl && selectedGame}
        <section class="controller-panel">
          <button type="button" class="back-button" on:click={() => (activeView = "library")}>
            <span class="i-material-symbols:arrow-back-rounded text-[20px]" />
            <span>一覧</span>
          </button>
          <div class="controller-kicker">Now Playing</div>
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
                : "i-material-symbols:pause-rounded"} text-[30px]`}
            />
            <span>{isPaused ? "記録再開" : "Pause"}</span>
          </button>

          <button
            type="button"
            class="shutter-button"
            disabled={connectionState !== "connected" || isSendingScreenshot}
            on:click={() => takeScreenshot()}
          >
            <span class="i-material-symbols:photo-camera-outline-rounded text-[36px]" />
            <span>{isSendingScreenshot ? "撮影中" : "スクショ"}</span>
          </button>

          <button
            type="button"
            class="textless-button"
            disabled={connectionState !== "connected" || isSendingScreenshot}
            on:click={() => takeScreenshot(true)}
          >
            <span class="i-material-symbols:visibility-off-outline-rounded text-[22px]" />
            <span>文字消しスクショ</span>
          </button>

        </section>
      {:else}
        <section class="empty-state controller-waiting">
          <span class="i-material-symbols:gamepad-outline-rounded text-[34px]" />
          <strong>起動中のゲームを待っています</strong>
          <small>PC側でLauncherg-Modからゲームを起動すると、自動でコントローラーに接続します。</small>
          {#if connectionState === "connected"}
            <button type="button" class="secondary-full-action" on:click={requestControlStatus}>
              状態を更新
            </button>
          {/if}
        </section>
      {/if}
    {/if}

    {#if showNowPlayingBar && selectedGame}
      <button type="button" class="now-playing-bar" on:click={openController}>
        <span class="i-material-symbols:gamepad-outline-rounded text-[24px]" />
        <span class="now-playing-copy">
          <strong>{selectedGame.title}</strong>
          <small>{isPaused ? "Pause中" : "コントローラーを開く"}</small>
        </span>
        <span class="i-material-symbols:chevron-right-rounded text-[22px]" />
      </button>
    {/if}

    {#if lastActionText}
      <div class="toast-line">{lastActionText}</div>
    {/if}
  </section>

  <nav class="bottom-nav" aria-label="スマホ連携">
    <button
      type="button"
      class:active={activeView === "library" || activeView === "detail"}
      on:click={() => (activeView = "library")}
    >
      <span class="i-material-symbols:format-list-bulleted-rounded text-[22px]" />
      <span>一覧</span>
    </button>
    <button
      type="button"
      class:active={activeView === "controller"}
      on:click={() => {
        activeView = "controller";
        requestControlStatus();
      }}
    >
      <span class="i-material-symbols:sports-esports-outline-rounded text-[22px]" />
      <span>操作</span>
    </button>
    <button
      type="button"
      class:active={activeView === "connect"}
      on:click={() => (activeView = "connect")}
    >
      <span class="i-material-symbols:wifi-tethering-rounded text-[22px]" />
      <span>接続</span>
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
    overflow-x: hidden;
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
    overflow-x: hidden;
    padding: 14px 14px calc(92px + env(safe-area-inset-bottom, 0px));
    display: grid;
    align-content: start;
    gap: 14px;
  }

  .content.controller-mode {
    padding-bottom: calc(92px + env(safe-area-inset-bottom, 0px));
  }

  .section,
  .library-view,
  .detail-panel,
  .controller-panel,
  .empty-state,
  .toast-line {
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(255 255 255 / 0.045);
  }


  .section,
  .library-view,
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
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
    padding: 12px 0 2px;
  }

  .filter-strip button {
    min-height: 36px;
    min-width: 0;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 999px;
    background: rgb(255 255 255 / 0.05);
    color: rgb(255 255 255 / 0.68);
    padding: 0 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    font-weight: 800;
  }

  .filter-strip button.active {
    border-color: rgb(94 201 142 / 0.55);
    background: rgb(94 201 142 / 0.16);
    color: #b7f3cb;
  }


  .library-toolbar {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 12px;
  }

  .search-shell {
    min-height: 46px;
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 14px;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(0 0 0 / 0.22);
    color: rgb(255 255 255 / 0.45);
    padding: 0 12px;
  }

  .search-shell .search-input {
    height: auto;
    min-width: 0;
    flex: 1;
    border: 0;
    background: transparent;
    padding: 0;
  }

  .game-card-list {
    display: grid;
    gap: 10px;
    margin-top: 12px;
  }

  .compact-list {
    margin-top: 12px;
  }

  .game-card {
    min-width: 0;
    width: 100%;
    display: flex;
    align-items: stretch;
    gap: 12px;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(255 255 255 / 0.045);
    color: white;
    padding: 10px;
    text-align: left;
    overflow: hidden;
  }

  .game-card.compact-card {
    min-height: 106px;
  }


  .game-thumb {
    position: relative;
    flex: 0 0 96px;
    width: 96px;
    aspect-ratio: 16 / 10;
    align-self: stretch;
    min-height: 92px;
    display: grid;
    place-items: center;
    overflow: hidden;
    border-radius: 7px;
    background:
      linear-gradient(135deg, rgb(94 201 142 / 0.18), transparent),
      rgb(255 255 255 / 0.06);
    color: rgb(255 255 255 / 0.34);
  }

  .game-thumb :global(img) {
    width: 100% !important;
    height: 100% !important;
    display: block !important;
    object-fit: cover !important;
    justify-self: stretch !important;
    align-self: stretch !important;
  }

  .detail-thumb :global(img) {
    width: 100% !important;
    height: auto !important;
    display: block !important;
    object-fit: contain !important;
  }

  .game-card-body {
    min-width: 0;
    flex: 1;
    align-self: center;
    text-align: left;
  }

  .game-card .game-title {
    display: -webkit-box;
    overflow: hidden;
    text-overflow: initial;
    white-space: normal;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-height: 1.35;
  }

  .game-card .game-meta {
    white-space: normal;
  }

  .meta-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-top: 8px;
  }

  .meta-chips span {
    min-height: 22px;
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    background: rgb(255 255 255 / 0.075);
    color: rgb(255 255 255 / 0.62);
    padding: 0 8px;
    font-size: 11px;
    font-weight: 800;
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

  .detail-kicker,
  .controller-kicker {
    margin-top: 8px;
    color: #9ae6b4;
    font-size: 12px;
    font-weight: 900;
  }

  .detail-thumb {
    width: 100%;
    max-width: 320px;
    min-height: 172px;
    margin: 14px auto 0 auto;
    display: flex;
    justify-content: center;
    align-items: center;
    overflow: hidden;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background:
      linear-gradient(135deg, rgb(94 201 142 / 0.16), transparent),
      rgb(255 255 255 / 0.055);
    color: rgb(255 255 255 / 0.32);
  }

  .detail-thumb.has-image {
    min-height: 0 !important;
    background: transparent !important;
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

  .primary-action,
  .secondary-action,
  .textless-button,
  .secondary-full-action {
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
  .textless-button,
  .secondary-full-action {
    background: rgb(255 255 255 / 0.06);
    color: rgb(255 255 255 / 0.9);
  }

  .secondary-action.paused-action,
  .pause-button.paused-action {
    border-color: rgb(245 158 11 / 0.5);
    background: rgb(245 158 11 / 0.18);
    color: #fbd38d;
  }

  .textless-button {
    width: 100%;
    margin-top: 10px;
  }

  .secondary-full-action {
    width: 100%;
    margin-top: 12px;
  }

  .memo-preview {
    margin-top: 14px;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(0 0 0 / 0.16);
    padding: 12px;
  }

  .detail-memo-editor {
    display: grid;
    gap: 10px;
    margin-top: 12px;
  }

  .detail-memo-editor textarea {
    min-height: 220px;
    width: 100%;
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

  .detail-memo-editor textarea::placeholder {
    color: rgb(255 255 255 / 0.36);
  }

  .detail-memo-editor button {
    min-height: 46px;
    border: 0;
    border-radius: 8px;
    background: white;
    color: #151515;
    font-weight: 900;
  }

  .connect-view {
    display: grid;
    gap: 14px;
  }

  .connect-card {
    min-height: 82px;
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 14px;
    border: 1px solid rgb(255 255 255 / 0.1);
    border-radius: 8px;
    background: rgb(255 255 255 / 0.045);
    padding: 12px;
  }

  .connect-card.connected {
    border-color: rgb(94 201 142 / 0.45);
  }

  .connect-card.offline {
    border-color: rgb(245 158 11 / 0.42);
  }

  .connect-card.error {
    border-color: rgb(252 129 129 / 0.42);
  }

  .connect-card div > strong,
  .connect-card div > span {
    display: block;
  }

  .connect-card div > strong {
    font-size: 15px;
    font-weight: 900;
  }

  .connect-card div > span {
    margin-top: 4px;
    color: rgb(255 255 255 / 0.52);
    font-size: 12px;
    font-weight: 700;
  }

  .connect-copy {
    margin-top: 12px;
    color: rgb(255 255 255 / 0.62);
    font-size: 13px;
    line-height: 1.65;
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

  .now-playing-bar {
    position: sticky;
    bottom: 0;
    min-height: 64px;
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    border: 1px solid rgb(94 201 142 / 0.4);
    border-radius: 8px;
    background: rgb(31 44 37 / 0.96);
    color: white;
    padding: 10px 12px;
    box-shadow: 0 12px 28px rgb(0 0 0 / 0.32);
  }

  .now-playing-copy {
    min-width: 0;
    flex: 1;
    text-align: left;
  }

  .now-playing-copy strong,
  .now-playing-copy small {
    display: block;
  }

  .now-playing-copy strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    font-weight: 900;
  }

  .now-playing-copy small {
    margin-top: 4px;
    color: #b7f3cb;
    font-size: 11px;
    font-weight: 800;
  }

  .empty-state {
    min-height: 132px;
    display: grid;
    place-items: center;
    gap: 8px;
    color: rgb(255 255 255 / 0.52);
    font-size: 13px;
    font-weight: 700;
    text-align: center;
    padding: 16px;
  }

  .controller-waiting {
    min-height: calc(100vh - 180px);
    align-content: center;
  }

  .controller-waiting strong,
  .controller-waiting small {
    display: block;
  }

  .controller-waiting strong {
    color: rgb(255 255 255 / 0.84);
    font-size: 17px;
    font-weight: 900;
  }

  .controller-waiting small {
    max-width: 280px;
    color: rgb(255 255 255 / 0.52);
    font-size: 12px;
    line-height: 1.6;
  }

  .controller-waiting .secondary-full-action {
    max-width: 220px;
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
    height: calc(72px + env(safe-area-inset-bottom, 0px));
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    padding-bottom: env(safe-area-inset-bottom, 0px);
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
