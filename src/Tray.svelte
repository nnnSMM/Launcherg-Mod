<script lang="ts">
  import { onMount } from "svelte";
  import { appWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { exit } from "@tauri-apps/api/process";

  let recentlyPlayed: { id: { value: number }; gamename: string }[] = [];

  onMount(async () => {
    // Fetch recently played games
    // This logic is moved from the backend `build_tray_menu`
    try {
      const allGames: any[] = await invoke("get_all_elements");
      allGames.sort((a, b) => {
        const dateA = a.last_play_at ? new Date(a.last_play_at).getTime() : 0;
        const dateB = b.last_play_at ? new Date(b.last_play_at).getTime() : 0;
        return dateB - dateA;
      });
      recentlyPlayed = allGames
        .filter((g) => g.last_play_at)
        .slice(0, 10)
        .map(g => ({ id: { value: g.id.value }, gamename: g.gamename }));
    } catch (e) {
      console.error("Failed to get recently played games:", e);
    }
  });

  async function launchGame(gameId: number) {
    try {
      await invoke("play_game", {
        id: gameId,
      });
      await appWindow.hide();
    } catch (e) {
      console.error(`Failed to launch game ${gameId}:`, e);
    }
  }

  async function launchShortcutGame() {
    try {
        await invoke("launch_shortcut_game");
        await appWindow.hide();
    } catch(e) {
        console.error("Failed to launch shortcut game:", e);
    }
  }

  async function quitApp() {
    await exit(0);
  }
</script>

<div class="tray-container">
  <div class="menu">
    <button class="menu-item" on:click={launchShortcutGame}>
      ショートカットのゲームを起動
    </button>
    <div class="separator"></div>
    <div class="submenu-label">最近プレイしたゲーム</div>
    {#each recentlyPlayed as game}
      <button class="menu-item" on:click={() => launchGame(game.id.value)}>
        {game.gamename}
      </button>
    {/each}
    <div class="separator"></div>
    <button class="menu-item" on:click={quitApp}> 終了 </button>
  </div>
</div>

<style>
  :global(body) {
    background-color: transparent;
    margin: 0;
    font-family: sans-serif;
  }

  .tray-container {
    background-color: #2a2a2e;
    border-radius: 8px;
    padding: 8px;
    color: #f0f0f0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    border: 1px solid #444;
  }

  .menu {
    display: flex;
    flex-direction: column;
  }

  .menu-item {
    background-color: transparent;
    border: none;
    color: #f0f0f0;
    padding: 8px 12px;
    text-align: left;
    width: 100%;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .menu-item:hover {
    background-color: #3c3c42;
  }

  .separator {
    height: 1px;
    background-color: #444;
    margin: 4px 0;
  }

  .submenu-label {
    padding: 8px 12px;
    font-size: 12px;
    color: #aaa;
  }
</style>
