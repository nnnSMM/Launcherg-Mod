<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { backgroundState } from "@/store/background";
  import Card from "@/components/UI/Card.svelte";
  import Checkbox from "@/components/UI/Checkbox.svelte";
  import { showErrorToast } from "@/lib/toast";
  import { SHOW_SENSITIVE_VNDB_SCREENSHOTS_SETTING_KEY } from "@/lib/useVndbScreenshots";

  let showSensitiveVndbScreenshots = false;
  let isLoading = true;

  onMount(async () => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });

    try {
      const savedShowSensitiveVndbScreenshots = await invoke<string | null>(
        "get_app_setting",
        {
          key: SHOW_SENSITIVE_VNDB_SCREENSHOTS_SETTING_KEY,
        },
      );
      showSensitiveVndbScreenshots =
        savedShowSensitiveVndbScreenshots === "true";
    } catch (error) {
      console.error("Error loading display settings:", error);
    } finally {
      isLoading = false;
    }
  });

  async function updateShowSensitiveVndbScreenshots() {
    if (isLoading) return;
    try {
      await invoke("set_app_setting", {
        key: SHOW_SENSITIVE_VNDB_SCREENSHOTS_SETTING_KEY,
        value: showSensitiveVndbScreenshots ? "true" : null,
      });
    } catch (error) {
      console.error("Error saving show_sensitive_vndb_screenshots:", error);
      showErrorToast(`表示設定の保存に失敗しました: ${error}`);
    }
  }

  $: showSensitiveVndbScreenshots, updateShowSensitiveVndbScreenshots();
</script>

<div class="p-4 text-text-primary space-y-4 h-full overflow-y-auto">
  <div class="flex items-center gap-2">
    <div class="i-material-symbols-display-settings-outline w-6 h-6" />
    <h1 class="text-2xl font-bold">表示設定</h1>
  </div>

  {#if isLoading}
    <p>設定を読み込み中...</p>
  {:else}
    <div class="space-y-6">
      <Card className="relative z-0">
        <div class="flex items-center gap-2 mb-2">
          <div class="i-material-symbols-image-outline w-5 h-5" />
          <h2 class="text-lg font-medium">ホーム画面のプレビュー</h2>
        </div>
        <!-- svelte-ignore a11y-label-has-associated-control -->
        <label class="flex gap-3 cursor-pointer items-start">
          <Checkbox bind:value={showSensitiveVndbScreenshots} />
          <div>
            <div class="text-(text-primary body) font-medium">
              刺激の強いVNDBスクリーンショットも表示する
            </div>
            <div class="text-(text-tertiary body2)">
              オフの場合、性的・暴力表現フラグが高い画像はホームのホバープレビューから除外します。
            </div>
          </div>
        </label>
      </Card>
    </div>
  {/if}
</div>
