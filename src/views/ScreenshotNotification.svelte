<script lang="ts">
  import { onMount } from "svelte";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const win = getCurrentWindow();
  const params = new URLSearchParams(window.location.search);

  const restoreHwnd = params.get("restoreHwnd") ?? "";
  const filePath = params.get("path") ?? "";

  let screenshotUrl: string | null = filePath ? convertFileSrc(filePath) : null;
  let visible = true;
  let didImageFail = false;
  let didRestoreForeground = false;

  const restoreForeground = () => {
    if (didRestoreForeground || !restoreHwnd || restoreHwnd === "0") {
      return;
    }
    didRestoreForeground = true;
    void invoke("restore_foreground_window", { hwnd: restoreHwnd });
  };

  const closeAfterFade = () => {
    restoreForeground();
    visible = false;
    window.setTimeout(() => {
      void win.close();
    }, 40);
  };

  onMount(() => {
    window.setTimeout(restoreForeground, 0);
    const timer = window.setTimeout(closeAfterFade, 3600);
    return () => window.clearTimeout(timer);
  });
</script>

<main class="h-full w-full overflow-hidden p-1 font-sans text-white select-none">
  {#if visible}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <button
      type="button"
      tabindex="-1"
      class="notification-shell"
      on:pointerdown|preventDefault={restoreForeground}
      on:click={closeAfterFade}
      aria-label="スクリーンショット保存通知を閉じる"
    >
      <div class="preview-frame">
        {#if screenshotUrl && !didImageFail}
          <img
            src={screenshotUrl}
            alt=""
            class="preview-image"
            on:error={() => {
              didImageFail = true;
            }}
          />
        {:else}
          <div class="preview-placeholder">IMG</div>
        {/if}
      </div>
      <div class="min-w-0 flex-1 text-left">
        <div class="caption">スクリーンショットを保存しました</div>
      </div>
    </button>
  {/if}
</main>

<style>
  .notification-shell {
    display: flex;
    width: 100%;
    height: 100%;
    align-items: center;
    gap: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 6px;
    background: rgba(19, 22, 28, 0.94);
    box-shadow:
      0 12px 32px rgba(0, 0, 0, 0.42),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
    padding: 7px;
    color: white;
    outline: none;
    backdrop-filter: blur(10px);
  }

  .preview-frame {
    width: 92px;
    height: 52px;
    flex: 0 0 auto;
    overflow: hidden;
    border-radius: 3px;
    background: #05070a;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  }

  .preview-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .preview-placeholder {
    display: flex;
    width: 100%;
    height: 100%;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.36);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0;
  }

  .caption {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0;
    line-height: 1.3;
  }
</style>
