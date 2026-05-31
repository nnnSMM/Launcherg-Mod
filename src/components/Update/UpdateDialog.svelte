<script lang="ts">
  import { onDestroy } from "svelte";
  import Button from "@/components/UI/Button.svelte";
  import { appUpdate } from "@/store/update";

  let panelElement: HTMLDivElement | null = null;

  $: updateInfo = $appUpdate.update;
  $: isInstalling =
    $appUpdate.status === "installing" || $appUpdate.status === "installed";

  const handlePointerDown = (event: PointerEvent) => {
    if (!$appUpdate.isDialogOpen) {
      return;
    }
    const target = event.target;
    if (!(target instanceof Node)) {
      return;
    }
    if (panelElement?.contains(target)) {
      return;
    }
    if ((target as HTMLElement).closest("[data-update-badge]")) {
      return;
    }
    appUpdate.closeDialog();
  };

  window.addEventListener("pointerdown", handlePointerDown, true);

  onDestroy(() => {
    window.removeEventListener("pointerdown", handlePointerDown, true);
  });
</script>

{#if $appUpdate.isDialogOpen && updateInfo}
  <div
    bind:this={panelElement}
    class="fixed left-[210px] top-9 z-[70] w-72 rounded-md border border-solid border-border-primary bg-bg-primary p-3 shadow-lg"
  >
    <div class="flex items-start gap-3">
      <div class="h-8 w-8 flex shrink-0 items-center justify-center rounded-full bg-[#0284c7] text-white">
        <div class="i-material-symbols-system-update-alt-rounded text-lg" />
      </div>
      <div class="min-w-0">
        <div class="text-body2 font-medium text-text-primary">更新があります</div>
      </div>
    </div>

    {#if $appUpdate.installMessage}
      <div class="mt-3 rounded border border-solid border-border-primary bg-bg-secondary p-2">
        <div class="text-caption text-text-primary">
          {$appUpdate.installMessage}
        </div>
        {#if $appUpdate.installProgress !== null}
          <div class="mt-2 h-1.5 overflow-hidden rounded bg-bg-tertiary">
            <div
              class="h-full bg-[#0284c7] transition-all"
              style="width: {Math.round($appUpdate.installProgress * 100)}%;"
            />
          </div>
        {/if}
      </div>
    {/if}

    {#if $appUpdate.error}
      <div class="mt-3 rounded border border-solid border-accent-error bg-bg-secondary p-2 text-caption text-accent-error">
        {$appUpdate.error}
      </div>
    {/if}

    <div class="mt-3 grid grid-cols-2 gap-2">
      <Button
        text="demoで試す"
        leftIcon="i-material-symbols-open-in-new-rounded"
        disabled={isInstalling}
        on:click={() => appUpdate.openDemoPage()}
      />
      <Button
        text="アップデート"
        variant="primary"
        leftIcon="i-material-symbols-download-rounded"
        disabled={isInstalling || !updateInfo.canInstall}
        tooltip={!updateInfo.canInstall
          ? { content: "demo または mock では実行できません。" }
          : undefined}
        on:click={() => appUpdate.installUpdate()}
      />
    </div>
  </div>
{/if}
