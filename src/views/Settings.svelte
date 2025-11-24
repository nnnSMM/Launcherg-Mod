<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { backgroundState } from "@/store/background";
  import type { CollectionElement } from "../lib/types";
  import Button from "../components/UI/Button.svelte";
  import Select from "../components/UI/Select.svelte";
  import Input from "../components/UI/Input.svelte";
  import type { Option } from "@/lib/filter";
  import Card from "@/components/UI/Card.svelte";
  import { showErrorToast, showInfoToast } from "@/lib/toast";

  let games: CollectionElement[] = [];
  let gameOptions: Option<number>[] = [];
  let selectedGameId: number = 0;
  let shortcutKey: string = "";
  let pauseShortcutKey: string = "";
  let isLoading = true;
  let isSaving = false;

  onMount(async () => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });
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

      const savedPauseShortcutKey = await invoke<string>("get_app_setting", {
        key: "pause_shortcut_key",
      });
      if (savedPauseShortcutKey) {
        pauseShortcutKey = savedPauseShortcutKey;
      }
    } catch (error) {
      console.error("Error loading settings:", error);
    } finally {
      isLoading = false;
    }
  });

  function toggleModifier(
    currentValue: string,
    modifier: "Ctrl" | "Alt" | "Shift",
  ): string {
    let parts = currentValue.split("+").map((p) => p.trim());
    parts = parts.filter((p) => p !== "");

    if (parts.includes(modifier)) {
      parts = parts.filter((p) => p !== modifier);
    } else {
      parts.push(modifier);
    }

    const order: Record<string, number> = { Ctrl: 0, Alt: 1, Shift: 2 };
    parts.sort((a, b) => {
      const orderA = order[a] ?? 3;
      const orderB = order[b] ?? 3;
      return orderA - orderB;
    });

    return parts.join("+");
  }

  async function saveSettings() {
    if (isSaving) {
      return;
    }
    isSaving = true;
    try {
      const gameIdToSave =
        selectedGameId === 0 ? null : selectedGameId.toString();
      await invoke("set_app_setting", {
        key: "shortcut_game_id",
        value: gameIdToSave,
      });

      const keyToSave = shortcutKey === "" ? null : shortcutKey;
      await invoke("update_shortcut_registration", {
        newShortcutKey: keyToSave,
      });

      const pauseKeyToSave = pauseShortcutKey === "" ? null : pauseShortcutKey;
      await invoke("update_pause_shortcut_registration", {
        newShortcutKey: pauseKeyToSave,
      });

      showInfoToast("設定を保存しました");
    } catch (error) {
      console.error("Error saving settings:", error);
      showErrorToast(`設定の保存に失敗しました: ${error}`);
    } finally {
      isSaving = false;
    }
  }
</script>

<div class="p-4 text-text-primary space-y-4">
  <div class="flex items-center gap-2">
    <div class="i-material-symbols-settings-outline w-6 h-6" />
    <h1 class="text-2xl font-bold">ショートカット設定</h1>
  </div>

  {#if isLoading}
    <p>設定を読み込み中...</p>
  {:else}
    <div class="space-y-6">
      <Card className="relative z-20">
        <div class="flex items-center gap-2 mb-2">
          <div class="i-material-symbols-sports-esports-outline w-5 h-5" />
          <h2 class="text-lg font-medium">起動するゲーム</h2>
        </div>
        <p class="text-sm text-text-secondary mb-4">
          ショートカットで起動するゲームを選択してください。「None」を選択すると、ショートカットは無効になります。
        </p>
        <Select
          options={gameOptions}
          bind:value={selectedGameId}
          title="ゲームを選択"
          enableFilter={true}
          filterPlaceholder="ゲームを検索..."
        />
      </Card>

      <Card className="relative z-10">
        <div class="flex items-center gap-2 mb-2">
          <div class="i-material-symbols-keyboard-outline w-5 h-5" />
          <h2 class="text-lg font-medium">ゲーム起動用ショートカットキー</h2>
        </div>
        <p class="text-sm text-text-secondary mb-4">
          ゲームを起動するためのショートカットキーを定義します。有効なキーの組み合わせについては、<a
            href="https://tauri.app/v1/api/js/globalshortcut"
            target="_blank"
            class="text-accent-accent hover:underline">Tauriのドキュメント</a
          >を参照してください。
        </p>
        <Input bind:value={shortcutKey} placeholder="例: Ctrl+Shift+L" />
        <div class="flex gap-2 mt-2">
          <Button
            text="Ctrl"
            on:click={() => (shortcutKey = toggleModifier(shortcutKey, "Ctrl"))}
          />
          <Button
            text="Alt"
            on:click={() => (shortcutKey = toggleModifier(shortcutKey, "Alt"))}
          />
          <Button
            text="Shift"
            on:click={() => (shortcutKey = toggleModifier(shortcutKey, "Shift"))}
          />
        </div>
      </Card>

      <Card className="relative z-0">
        <div class="flex items-center gap-2 mb-2">
          <div class="i-material-symbols-pause-circle-outline w-5 h-5" />
          <h2 class="text-lg font-medium">Pause用ショートカットキー</h2>
        </div>
        <p class="text-sm text-text-secondary mb-4">
          ゲームプレイ時間の計測を一時停止するためのショートカットキーを定義します。ゲームを起動中でも、休憩や離席などで実際にプレイしていない時間がある場合、このショートカットキーで計測を一時停止できます。一時停止すると画面上にオーバーレイが表示され、再開するには画面をクリックするか、再度同じキーを押してください。有効なキーの組み合わせについては、<a
            href="https://tauri.app/v1/api/js/globalshortcut"
            target="_blank"
            class="text-accent-accent hover:underline">Tauriのドキュメント</a
          >を参照してください。
        </p>
        <Input bind:value={pauseShortcutKey} placeholder="例: Ctrl+Shift+P" />
        <div class="flex gap-2 mt-2">
          <Button
            text="Ctrl"
            on:click={() => (pauseShortcutKey = toggleModifier(pauseShortcutKey, "Ctrl"))}
          />
          <Button
            text="Alt"
            on:click={() => (pauseShortcutKey = toggleModifier(pauseShortcutKey, "Alt"))}
          />
          <Button
            text="Shift"
            on:click={() => (pauseShortcutKey = toggleModifier(pauseShortcutKey, "Shift"))}
          />
        </div>
      </Card>

      <div class="flex justify-end">
        <Button
          on:click={saveSettings}
          text={isSaving ? "保存中..." : "設定を保存"}
          disabled={isSaving}
        />
      </div>
    </div>
  {/if}
</div>
