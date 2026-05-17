<script lang="ts">
  import { onMount } from "svelte";
  import { backgroundState } from "@/store/background";
  import Card from "@/components/UI/Card.svelte";
  import { showErrorToast } from "@/lib/toast";
  import { theme, type AppTheme } from "@/store/theme";

  let isLoading = true;
  let savingTheme = false;

  onMount(() => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });

    isLoading = false;
  });

  async function updateTheme(nextTheme: AppTheme) {
    if (savingTheme || $theme === nextTheme) return;
    savingTheme = true;
    try {
      await theme.set(nextTheme);
    } catch (error) {
      console.error("Error saving theme:", error);
      showErrorToast(`テーマ設定の保存に失敗しました: ${error}`);
    } finally {
      savingTheme = false;
    }
  }
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
          <div class="i-material-symbols-palette-outline w-5 h-5" />
          <h2 class="text-lg font-medium">テーマ</h2>
        </div>
        <div class="flex flex-wrap gap-2">
          <button
            type="button"
            class="h-9 px-4 rounded border border-border-primary transition-colors flex items-center gap-2 {$theme ===
            'dark'
              ? 'bg-accent-primary text-text-white'
              : 'bg-bg-button text-text-primary hover:bg-bg-button-hover'}"
            on:click={() => updateTheme("dark")}
            disabled={savingTheme}
          >
            <div class="i-material-symbols-dark-mode-outline w-4 h-4" />
            <span class="text-sm font-medium">ダーク</span>
          </button>
          <button
            type="button"
            class="h-9 px-4 rounded border border-border-primary transition-colors flex items-center gap-2 {$theme ===
            'light'
              ? 'bg-accent-primary text-text-white'
              : 'bg-bg-button text-text-primary hover:bg-bg-button-hover'}"
            on:click={() => updateTheme("light")}
            disabled={savingTheme}
          >
            <div class="i-material-symbols-light-mode-outline w-4 h-4" />
            <span class="text-sm font-medium">ライト</span>
          </button>
        </div>
        <div class="text-text-tertiary text-body2 mt-3">
          アプリ全体の配色を切り替えます。画像上の文字は見やすさを優先して暗いオーバーレイを使います。
        </div>
      </Card>
    </div>
  {/if}
</div>
