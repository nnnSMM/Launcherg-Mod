<script lang="ts">
  import { systemStatus } from "@/store/systemStatus";
  import { fade } from "svelte/transition";

  $: ({ isInitializing, message } = $systemStatus);
</script>

{#if isInitializing}
  <div
    transition:fade={{ duration: 300 }}
    class="fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-bg-backdrop/80 backdrop-blur-md select-none"
  >
    <div class="relative flex flex-col items-center max-w-md px-8 text-center">
      <!-- Loading Spinner -->
      <div class="relative mb-8 h-16 w-16">
        <div class="absolute inset-0 rounded-full border-4 border-accent-accent/20"></div>
        <div
          class="absolute inset-0 animate-spin rounded-full border-4 border-accent-accent border-t-transparent"
        ></div>
      </div>

      <!-- Message Header -->
      <h2 class="mb-3 text-h2 font-bold text-text-primary tracking-tight">
        初期化しています
      </h2>

      <!-- Progress Message -->
      <p class="text-body2 text-text-secondary leading-relaxed">
        {message || "しばらくお待ちください..."}
      </p>

      <!-- Sub text -->
      <p class="mt-8 text-caption text-text-tertiary">
        ゲームデータベースの準備が完了するまで、アプリケーションの操作を制限しています。
      </p>
    </div>
  </div>
{/if}

<style lang="scss">
  /* 背景のクリックを完全に無効化するための追加設定 */
  div {
    pointer-events: all;
  }
</style>
