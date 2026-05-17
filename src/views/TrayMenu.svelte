<script lang="ts">
  import { onMount } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { CollectionElement } from "@/lib/types";
  import {
    commandGetAllElements,
    commandGetAppSetting,
    commandHideTrayMenu,
    commandLaunchShortcutGame,
    commandPlayGame,
    commandQuitApp,
    commandShowMainWindow,
  } from "@/lib/command";

  let recentGames: CollectionElement[] = [];
  let shortcutGame: CollectionElement | null = null;
  let isLoading = true;
  let isLaunching = false;

  const loadMenu = async () => {
    isLoading = true;
    try {
      const [games, shortcutGameId] = await Promise.all([
        commandGetAllElements(),
        commandGetAppSetting("shortcut_game_id"),
      ]);

      recentGames = games
        .filter((game) => game.lastPlayAt)
        .sort((a, b) => {
          const aTime = new Date(a.lastPlayAt ?? 0).getTime();
          const bTime = new Date(b.lastPlayAt ?? 0).getTime();
          return bTime - aTime;
        })
        .slice(0, 8);

      const parsedShortcutId = Number(shortcutGameId);
      shortcutGame = Number.isFinite(parsedShortcutId)
        ? games.find((game) => game.id === parsedShortcutId) ?? null
        : null;
    } finally {
      isLoading = false;
    }
  };

  const hideMenu = async () => {
    await commandHideTrayMenu();
  };

  const openMainWindow = async () => {
    await commandShowMainWindow();
  };

  const launchShortcutGame = async () => {
    if (!shortcutGame || isLaunching) return;
    isLaunching = true;
    try {
      await commandHideTrayMenu();
      await commandLaunchShortcutGame();
    } finally {
      isLaunching = false;
    }
  };

  const playRecentGame = async (game: CollectionElement) => {
    if (isLaunching) return;
    isLaunching = true;
    try {
      await commandHideTrayMenu();
      await commandPlayGame(game.id, false);
    } finally {
      isLaunching = false;
    }
  };

  const quit = async () => {
    await commandQuitApp();
  };

  const thumbnailSrc = (game: CollectionElement) =>
    game.thumbnail
      ? `${convertFileSrc(game.thumbnail)}?v=${game.updatedAt}`
      : "/images/dummy_thumbnail.svg";

  const iconSrc = (game: CollectionElement) =>
    game.icon ? `${convertFileSrc(game.icon)}?v=${game.updatedAt}` : "/icon.png";

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      void hideMenu();
    }
  };

  onMount(() => {
    void loadMenu();
  });
</script>

<svelte:window on:focus={() => void loadMenu()} on:keydown={handleKeydown} />

<main class="tray-root h-screen w-screen overflow-hidden font-sans select-none">
  <section
    class="tray-surface h-full w-full flex flex-col rounded-lg border border-border-primary bg-bg-primary/92 backdrop-blur-xl shadow-[0_16px_48px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,255,255,0.05)]"
  >
    <header
      class="flex items-center gap-2.5 min-w-0 px-3 pt-2.5 pb-2 border-b border-border-primary"
    >
      <img
        src="/icon.png"
        alt=""
        class="h-8 w-8 rounded-md shadow-sm ring-1 ring-white/10 shrink-0"
      />
      <div class="min-w-0">
        <div class="text-text-primary text-body3 font-bold leading-tight tracking-tight">
          Launcherg
        </div>
        <div class="text-text-tertiary text-caption truncate leading-snug">
          クイックメニュー
        </div>
      </div>
    </header>

    <div class="px-2.5 pt-2 pb-1.5 flex flex-col gap-1.5">
      <button
        type="button"
        class="tray-primary-cta focus-ring"
        on:click={openMainWindow}
      >
        <span class="i-material-symbols-open-in-new-rounded h-4 w-4 shrink-0 opacity-90" />
        <span class="truncate">メインウィンドウを開く</span>
      </button>

      {#if shortcutGame}
        <button
          type="button"
          class="tray-shortcut-card focus-ring"
          disabled={isLaunching}
          on:click={launchShortcutGame}
          title={`${shortcutGame.gamename} を起動（ショートカット）`}
        >
          <div class="tray-shortcut-thumb-wrap">
            <img
              src={thumbnailSrc(shortcutGame)}
              alt=""
              class="h-full w-full object-cover"
              on:error={(event) => {
                if (event.currentTarget instanceof HTMLImageElement) {
                  event.currentTarget.src = "/images/dummy_thumbnail.svg";
                }
              }}
            />
          </div>
          <div class="min-w-0 flex-1 text-left">
            <span class="block text-caption font-semibold text-accent-accent tracking-wide">
              ショートカット
            </span>
            <span class="block text-text-primary text-body3 font-semibold truncate mt-0.5">
              {shortcutGame.gamename}
            </span>
          </div>
          <span
            class="i-material-symbols-play-circle-rounded h-6 w-6 shrink-0 text-accent-accent"
          />
        </button>
      {:else}
        <div
          class="rounded-md border border-dashed border-border-primary bg-bg-secondary/50 px-2.5 py-2 text-text-tertiary text-caption text-center leading-snug"
        >
          ショートカットゲームは未設定です（メインの設定から指定できます）
        </div>
      {/if}
    </div>

    <div
      class="section-head px-3 pt-0.5 pb-1.5 flex items-center justify-between gap-2 text-text-tertiary text-caption font-semibold tracking-wide"
    >
      <span>最近プレイしたゲーム</span>
      {#if isLoading}
        <span class="normal-case font-normal opacity-80">読み込み中…</span>
      {/if}
    </div>

    <div class="recent-scroll flex-1 min-h-0 overflow-y-auto px-1.5 pb-1.5">
      {#if !isLoading && recentGames.length === 0}
        <div
          class="flex flex-col items-center justify-center gap-1.5 py-7 px-3 text-text-tertiary text-body3 text-center"
        >
          <span class="i-material-symbols-history-rounded h-7 w-7 opacity-60" />
          <span>まだ履歴がありません</span>
        </div>
      {:else}
        {#each recentGames as game (game.id)}
          <!-- button の UA 色でテキストが白固定になる WebView 対策で div + role -->
          <div
            role="button"
            tabindex={isLaunching ? -1 : 0}
            class="tray-recent-row focus-ring"
            class:tray-recent-row--busy={isLaunching}
            aria-disabled={isLaunching}
            title={`${game.gamename} を起動`}
            on:click={() => {
              if (!isLaunching) void playRecentGame(game);
            }}
            on:keydown={(e) => {
              if (isLaunching) return;
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                void playRecentGame(game);
              }
            }}
          >
            <img
              src={iconSrc(game)}
              alt=""
              class="h-8 w-8 rounded-md object-cover bg-bg-secondary ring-1 ring-white/5 flex-shrink-0"
              on:error={(event) => {
                if (event.currentTarget instanceof HTMLImageElement) {
                  event.currentTarget.src = "/icon.png";
                }
              }}
            />
            <span class="min-w-0 flex-1 text-left">
              <span class="tray-recent-name truncate">
                {game.gamename}
              </span>
              <span class="tray-recent-meta truncate">
                {game.brandname || "—"}
              </span>
            </span>
            <span class="i-material-symbols-play-arrow-rounded tray-play-glyph h-4 w-4 shrink-0" />
          </div>
        {/each}
      {/if}
    </div>

    <footer class="mt-auto border-t border-border-primary px-2.5 py-2 bg-bg-backdrop/40">
      <button
        type="button"
        class="tray-quit focus-ring w-full justify-center"
        on:click={quit}
      >
        <span class="i-material-symbols-power-settings-new-rounded h-4 w-4" />
        <span>Launcherg を終了</span>
      </button>
    </footer>
  </section>
</main>

<style lang="scss">
  .tray-root {
    background: transparent;
  }

  .tray-primary-cta {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: 100%;
    min-height: 2.25rem;
    padding: 0 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: #e6edf3;
    background: linear-gradient(180deg, #444c56 0%, #373e47 100%);
    border: 1px solid rgba(205, 217, 229, 0.14);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
    transition:
      background-color 160ms ease,
      border-color 160ms ease,
      box-shadow 160ms ease,
      transform 120ms ease;

    &:hover {
      background: linear-gradient(180deg, #4e5662 0%, #444c56 100%);
      border-color: rgba(72, 122, 249, 0.35);
      box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.08),
        0 0 0 1px rgba(72, 122, 249, 0.15);
    }

    &:active {
      transform: scale(0.99);
    }
  }

  .tray-shortcut-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    min-height: 3.5rem;
    padding: 0.35rem 0.5rem;
    text-align: left;
    border-radius: 0.375rem;
    border: 1px solid rgba(72, 122, 249, 0.28);
    background: linear-gradient(
      135deg,
      rgba(72, 122, 249, 0.14) 0%,
      rgba(22, 27, 34, 0.85) 100%
    );
    transition:
      background 160ms ease,
      border-color 160ms ease,
      box-shadow 160ms ease,
      transform 120ms ease;

    &:hover:not(:disabled) {
      border-color: rgba(72, 122, 249, 0.55);
      box-shadow: 0 0 0 1px rgba(72, 122, 249, 0.12);
      background: linear-gradient(
        135deg,
        rgba(72, 122, 249, 0.2) 0%,
        rgba(22, 27, 34, 0.92) 100%
      );
    }

    &:active:not(:disabled) {
      transform: scale(0.995);
    }

    &:disabled {
      opacity: 0.65;
      cursor: not-allowed;
    }
  }

  .tray-shortcut-thumb-wrap {
    width: 4.25rem;
    height: 3rem;
    flex-shrink: 0;
    border-radius: 0.375rem;
    overflow: hidden;
    background: #21262d;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.06);
  }

  .tray-recent-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    min-height: 2.625rem;
    padding: 0.3rem 0.45rem;
    margin-bottom: 0.0625rem;
    border-radius: 0.375rem;
    border: 1px solid transparent;
    box-sizing: border-box;
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    transition:
      background-color 160ms ease,
      border-color 160ms ease,
      transform 120ms ease;

    &:hover:not(.tray-recent-row--busy) {
      background: rgba(205, 217, 229, 0.06);
      border-color: rgba(205, 217, 229, 0.08);
    }

    &:hover:not(.tray-recent-row--busy) .tray-recent-name {
      color: #cdd9e5 !important;
    }

    &:hover:not(.tray-recent-row--busy) .tray-recent-meta {
      color: #9dacbc !important;
    }

    &:active:not(.tray-recent-row--busy) {
      transform: scale(0.99);
    }

    &--busy {
      opacity: 0.55;
      cursor: default;
      pointer-events: none;
    }
  }

  .tray-recent-name {
    display: block;
    font-size: 0.8rem;
    line-height: 1.45;
    font-weight: 500;
    color: #8b949e !important;
  }

  .tray-recent-meta {
    display: block;
    margin-top: 1px;
    font-size: 0.72rem;
    line-height: 1.4;
    color: #6e7681 !important;
  }

  /* 三角アイコンはアクセント青のまま（変更不要とのこと） */
  .tray-play-glyph {
    color: #487af9 !important;
    opacity: 0.92;
    transition:
      opacity 160ms ease,
      filter 160ms ease;
  }

  .tray-recent-row:hover:not(.tray-recent-row--busy) .tray-play-glyph {
    opacity: 1;
    filter: brightness(1.08);
  }

  .tray-quit {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    min-height: 2rem;
    padding: 0 0.65rem;
    border-radius: 0.375rem;
    font-size: 0.72rem;
    font-weight: 700;
    color: #adbac7;
    background: rgba(48, 54, 61, 0.65);
    border: 1px solid rgba(205, 217, 229, 0.1);
    transition:
      background-color 160ms ease,
      border-color 160ms ease,
      color 160ms ease,
      transform 120ms ease;

    &:hover {
      color: #fdaeb7;
      background: rgba(234, 78, 96, 0.14);
      border-color: rgba(234, 78, 96, 0.38);
    }

    &:active {
      transform: scale(0.99);
    }
  }

  .recent-scroll {
    scrollbar-gutter: stable;

    &::-webkit-scrollbar {
      width: 0.35rem;
    }

    &::-webkit-scrollbar-thumb {
      background-color: #55606d;
      border-radius: 999px;
    }

    &::-webkit-scrollbar-thumb:hover {
      background-color: #768390;
    }
  }
</style>
