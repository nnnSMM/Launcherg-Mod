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

      alert("設定を保存しました");
    } catch (error) {
      console.error("Error saving settings:", error);
      alert(`設定の保存に失敗しました: ${error}`);
    }
  }
</script>

<div class="p-4 text-text-primary">
  <h1 class="text-2xl font-bold mb-4">ショートカット設定</h1>

  {#if isLoading}
    <p>設定を読み込み中...</p>
  {:else}
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium mb-1">
          ショートカットで起動するゲーム
        </label>
        <Select
          options={gameOptions}
          bind:value={selectedGameId}
          title="ゲームを選択"
          enableFilter={true}
          filterPlaceholder="ゲームを検索..."
        />
        <p class="mt-1 text-sm">
          ショートカットで起動するゲームを選択してください。
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">
          ショートカットキー
        </label>
        <Input
          bind:value={shortcutKey}
          placeholder="例: CommandOrControl+Shift+L"
        />
        <p class="mt-1 text-sm">
          グローバルショートカットを定義します。CommandOrControl, Shift,
          Altなどの修飾キーが使えます。有効なアクセラレータ文字列については、<a
            href="https://tauri.app/v1/api/js/globalshortcut"
            target="_blank"
            class="text-blue-500 hover:underline">Tauriのドキュメント</a
          >を参照してください。
        </p>
      </div>

      <div>
        <Button on:click={saveSettings} text="設定を保存" />
      </div>
    </div>
  {/if}
</div>
