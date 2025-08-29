<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import type { CollectionElement } from "../lib/types";
  import Button from "../components/UI/Button.svelte";
  import Select from "../components/UI/Select.svelte";
  import Input from "../components/UI/Input.svelte";

  let games: CollectionElement[] = [];
  let selectedGameId: string = "";
  let shortcutKey: string = "";
  let isLoading = true;

  onMount(async () => {
    try {
      games = await invoke("get_all_elements");
      const savedGameId = await invoke<string>("get_app_setting", { key: "shortcut_game_id" });
      if (savedGameId) {
        selectedGameId = savedGameId;
      }
      const savedShortcutKey = await invoke<string>("get_app_setting", { key: "shortcut_key" });
      if (savedShortcutKey) {
        shortcutKey = savedShortcutKey;
      }
    } catch (error) {
      console.error("Error loading settings:", error);
    } finally {
      isLoading = false;
    }
  });

  async function saveSettings() {
    try {
      await invoke("set_app_setting", { key: "shortcut_game_id", value: selectedGameId });
      await invoke("set_app_setting", { key: "shortcut_key", value: shortcutKey });
      alert("Settings saved successfully!");
    } catch (error) {
      console.error("Error saving settings:", error);
      alert("Failed to save settings.");
    }
  }
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Shortcut Settings</h1>

  {#if isLoading}
    <p>Loading settings...</p>
  {:else}
    <div class="space-y-4">
      <div>
        <label for="game-select" class="block text-sm font-medium text-gray-700">
          Shortcut Game
        </label>
        <Select id="game-select" bind:value={selectedGameId} class="mt-1 block w-full">
          <option value="">None</option>
          {#each games as game (game.id)}
            <option value={game.id}>{game.gamename}</option>
          {/each}
        </Select>
        <p class="mt-1 text-sm text-gray-500">Select a game to launch with the shortcut.</p>
      </div>

      <div>
        <label for="shortcut-key" class="block text-sm font-medium text-gray-700">
          Shortcut Key
        </label>
        <Input
          id="shortcut-key"
          bind:value={shortcutKey}
          placeholder="e.g., CommandOrControl+Shift+L"
          class="mt-1 block w-full"
        />
        <p class="mt-1 text-sm text-gray-500">
          Define the global shortcut. Use modifiers like CommandOrControl, Shift, Alt. See <a
            href="https://tauri.app/v1/api/js/globalshortcut"
            target="_blank"
            class="text-blue-500 hover:underline">Tauri docs</a
          > for valid accelerator strings.
        </p>
      </div>

      <div>
        <Button on:click={saveSettings}>Save Settings</Button>
      </div>
    </div>
  {/if}
</div>
