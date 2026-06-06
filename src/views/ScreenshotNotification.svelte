<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { commandGetGameScreenshots } from "@/lib/command";

  const win = getCurrentWindow();
  const params = new URLSearchParams(window.location.search);
  const gameId = Number(params.get("gameId") ?? 0);
  const filename = params.get("filename") ?? "";

  let screenshotUrl: string | null = null;
  let visible = false;

  const closeAfterFade = () => {
    visible = false;
    window.setTimeout(() => {
      void win.close();
    }, 260);
  };

  onMount(async () => {
    if (Number.isFinite(gameId) && gameId > 0 && filename) {
      try {
        const screenshots = await commandGetGameScreenshots(gameId);
        const target =
          screenshots.find((screenshot) => screenshot.filename.endsWith(filename)) ??
          screenshots[0];
        if (target) {
          screenshotUrl = convertFileSrc(target.thumbnailFilename ?? target.filename);
        }
      } catch (error) {
        console.error("Failed to load screenshot preview", error);
      }
    }

    await win.show();
    visible = true;
    window.setTimeout(closeAfterFade, 3600);
  });
</script>

<main class="h-full w-full overflow-hidden p-2 font-sans text-white select-none">
  {#if visible}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <button
      type="button"
      class="notification-shell"
      in:fly={{ x: 24, duration: 180 }}
      out:fade={{ duration: 220 }}
      on:click={closeAfterFade}
      aria-label="スクリーンショット保存通知を閉じる"
    >
      <div class="preview-frame">
        {#if screenshotUrl}
          <img src={screenshotUrl} alt="" class="preview-image" />
        {:else}
          <div class="preview-placeholder">IMG</div>
        {/if}
      </div>
      <div class="min-w-0 flex-1 text-left">
        <div class="caption">スクリーンショットを保存しました</div>
        <div class="filename">{filename || "Launcherg-Mod"}</div>
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
    gap: 12px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 6px;
    background: rgba(19, 22, 28, 0.94);
    box-shadow:
      0 12px 32px rgba(0, 0, 0, 0.42),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
    padding: 10px;
    color: white;
    outline: none;
    backdrop-filter: blur(10px);
  }

  .preview-frame {
    width: 104px;
    height: 58px;
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
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0;
    line-height: 1.3;
  }

  .filename {
    margin-top: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: rgba(221, 226, 234, 0.68);
    font-size: 11px;
    line-height: 1.25;
    letter-spacing: 0;
  }
</style>
