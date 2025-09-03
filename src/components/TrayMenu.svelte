<script lang="ts">
  import { onMount } from "svelte";
  import { appWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/tauri";
  import { exit } from "@tauri-apps/api/process";
  import type { CollectionElement } from "$lib/types";

  let recentGames: CollectionElement[] = [];
  let shortcutGameName: string | null = "ショートカットのゲームを起動";

  onMount(async () => {
    // Hide the window when it loses focus
    await appWindow.listen("tauri://blur", () => {
      appWindow.hide();
    });

    // Fetch shortcut game name
    try {
      const gameIdStr: string | null = await invoke("get_app_setting", { name: "shortcut_game_id" });
      if (gameIdStr) {
        const gameId = parseInt(gameIdStr, 10);
        const game: CollectionElement = await invoke("get_collection_element", { collectionElementId: gameId });
        if (game) {
          shortcutGameName = `${game.gamename} を起動`;
        }
      }
    } catch (e) {
      console.error("Failed to fetch shortcut game", e);
    }

    // Fetch recent games
    try {
      const allGames: CollectionElement[] = await invoke("get_all_elements");
      allGames.sort((a, b) => {
        if (b.last_play_at && a.last_play_at) {
          return new Date(b.last_play_at).getTime() - new Date(a.last_play_at).getTime();
        }
        return 0;
      });
      recentGames = allGames.filter(g => g.last_play_at).slice(0, 10);
    } catch (e) {
      console.error("Failed to fetch recent games", e);
    }
  });

  async function launchShortcutGame() {
    try {
      await invoke("launch_shortcut_game");
      await appWindow.hide();
    } catch (e) {
      console.error("Failed to launch shortcut game", e);
    }
  }

  async function playGame(id: number) {
    try {
      await invoke("play_game", { elementId: id });
      await appWindow.hide();
    } catch (e) {
      console.error(`Failed to play game ${id}`, e);
    }
  }

  async function quitApp() {
    await exit(0);
  }
</script>

<div class="tray-menu">
  <div class="menu-items">
    <div class="menu-section">
       <button class="menu-item" on:click={launchShortcutGame}>
        {shortcutGameName}
      </button>
    </div>
    <div class="separator"></div>
    {#if recentGames.length > 0}
      <div class="menu-section">
        <div class="section-title">最近プレイしたゲーム</div>
        {#each recentGames as game}
          <button class="menu-item" on:click={() => playGame(game.id.value)}>
            {game.gamename}
          </button>
        {/each}
      </div>
      <div class="separator"></div>
    {/if}
    <div class="menu-section">
      <button class="menu-item" on:click={quitApp}> 終了 </button>
    </div>
  </div>
</div>

<style>
  .tray-menu {
    background-color: rgba(255, 255, 255, 0.9);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    padding: 6px;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(0, 0, 0, 0.1);
    font-family: sans-serif;
    font-size: 14px;
    color: #333;
    overflow: hidden;
  }

  .menu-items {
    display: flex;
    flex-direction: column;
  }

  .menu-section {
    display: flex;
    flex-direction: column;
  }

  .section-title {
    font-size: 12px;
    font-weight: 600;
    color: #666;
    padding: 8px 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .menu-item {
    background: none;
    border: none;
    padding: 8px 12px;
    width: 100%;
    text-align: left;
    cursor: pointer;
    border-radius: 4px;
    color: #333;
    font-size: 14px;
  }

  .menu-item:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }

  .separator {
    height: 1px;
    background-color: rgba(0, 0, 0, 0.1);
    margin: 6px 0;
  }
</style>
