<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { CollectionElement } from "../lib/types";
  import Button from "../components/UI/Button.svelte";
  import Select from "../components/UI/Select.svelte";
  import Input from "../components/UI/Input.svelte";
  import type { Option } from "@/lib/filter";

  let games: CollectionElement[] = [];
  let gameOptions: Option<number>[] = [];
  let selectedGameId: number = 0;
  let shortcutKey: string = "";
  let isLoading = true;

  onMount(async () => {
    try {
      games = await invoke("get_all_elements");
      gameOptions = [
        { label: "None", value: 0 },
        ...games.map((g) => ({ label: g.gamename, value: g.id })),
      ];

      const savedGameIdStr = await invoke<string>("get_app_setting", {
        key: "shortcut_game_id",
      });
      if (savedGameIdStr) {
        selectedGameId = parseInt(savedGameIdStr, 10);
      } else {
        selectedGameId = 0;
      }
      const savedShortcutKey = await invoke<string>("get_app_setting", {
        key: "shortcut_key",
      });
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
      const gameIdToSave =
        selectedGameId === 0 ? null : selectedGameId.toString();
      await invoke("set_app_setting", {
        key: "shortcut_game_id",
        value: gameIdToSave,
      });
      await invoke("set_app_setting", {
        key: "shortcut_key",
        value: shortcutKey,
      });

      await invoke("update_shortcut_registration");

      alert("Settings saved successfully!");
    } catch (error) {
      console.error("Error saving settings:", error);
      alert(`Failed to save settings: ${error}`);
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
        <label class="block text-sm font-medium text-gray-700 mb-1">
          Shortcut Game
        </label>
        <Select
          options={gameOptions}
          bind:value={selectedGameId}
          title="Select a game"
          enableFilter={true}
          filterPlaceholder="Search games..."
        />
        <p class="mt-1 text-sm text-gray-500">Select a game to launch with the shortcut.</p>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">
          Shortcut Key
        </label>
        <Input
          bind:value={shortcutKey}
          placeholder="e.g., CommandOrControl+Shift+L"
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
