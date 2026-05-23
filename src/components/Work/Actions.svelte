<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import PlayButton from "@/components/Work/PlayButton.svelte";
  import {
    commandDeleteCollectionElement,
    commandOpenFolder,
    commandPlayGame,
    commandUpdateElementLike,
    commandUpsertCollectionElement,
    commandSetAppSetting,
  } from "@/lib/command";
  import { showErrorToast, showInfoToast } from "@/lib/toast";
  import { localStorageWritable } from "@/lib/utils";
  import ButtonCancel from "@/components/UI/ButtonCancel.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import APopover from "@/components/UI/APopover.svelte";
  import SettingPopover from "@/components/Work/SettingPopover.svelte";
  import ImportManually from "@/components/Sidebar/ImportManually.svelte";
  import { deleteTab, tabs, selected } from "@/store/tabs";
  import DeleteElement from "@/components/Work/DeleteElement.svelte";
  import type { AllGameCacheOne } from "@/lib/types";
  import OtherInformation from "@/components/Work/OtherInformation.svelte";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import QrCode from "@/components/Work/QRCode.svelte";
  import { startProcessMap } from "@/store/startProcessMap";
  import { enqueueGameScreenshotPrefetch } from "@/lib/useGameScreenshots";

  export let id: number;
  export let seiyaUrl: string;

  $: element = $sidebarCollectionElements.find((e) => e.id === id);

  $: isLike = !!element?.likeAt;
  $: isInstalled = !!(element?.exePath || element?.lnkPath);

  const isAdminRecord = localStorageWritable<Record<number, boolean>>(
    "play-admin-cache",
    {},
  );

  const setAsShortcutGame = async () => {
    try {
      await commandSetAppSetting("shortcut_game_id", id.toString());
      showInfoToast("ショートカットに設定しました。");
    } catch (e) {
      showErrorToast(e as string);
    }
  };

  const play = async (isAdmin: boolean | undefined) => {
    sidebarCollectionElements.refetch();
    if (isAdmin !== undefined) {
      isAdminRecord.update((v) => {
        v[id] = isAdmin;
        return v;
      });
    }
    let _isAdmin: boolean = isAdmin ?? false;
    if (isAdmin === undefined) {
      const cache = $isAdminRecord[id];
      if (cache) {
        _isAdmin = cache;
      }
    }
    try {
      const processId = await commandPlayGame(id, _isAdmin);
      startProcessMap.update((v) => {
        if (processId) {
          v[id] = processId;
        }
        return v;
      });
    } catch (e) {
      const errStr = e as string;
      if (errStr.includes("executable not found")) {
        showErrorToast(
          "実行ファイルが見つかりません。パスを設定し直してください。",
        );
        isOpenImportManually = true;
      } else {
        showErrorToast(errStr);
      }
    }
  };

  const handlePlayClick = (
    e: CustomEvent<{ isAdmin: boolean | undefined }>,
  ) => {
    if (isInstalled) {
      play(e.detail.isAdmin);
    } else {
      isOpenImportManually = true;
    }
  };

  const toggleLike = async () => {
    await commandUpdateElementLike(id, !isLike);
    sidebarCollectionElements.updateLike(id, !isLike);
  };

  let isOpenImportManually = false;
  const onChangeGame = async (arg: {
    exePath: string | null;
    lnkPath: string | null;
    gameCache: AllGameCacheOne;
  }) => {
    const isChangedGameId = id !== arg.gameCache.id;
    if (isChangedGameId) {
      await commandDeleteCollectionElement(id);
    }
    await commandUpsertCollectionElement(arg);
    await registerCollectionElementDetails();
    await sidebarCollectionElements.refetch();
    const imported = sidebarCollectionElements
      .value()
      .find((v) => v.id === arg.gameCache.id);
    if (imported) {
      enqueueGameScreenshotPrefetch([imported]);
    }
    if (isChangedGameId) {
      deleteTab($tabs[$selected].id);
    }
    isOpenImportManually = false;
  };

  let isOpenDelete = false;
  let isOpenOtherInformation = false;
  let isOpenQrCode = false;
</script>

{#if element}
  <div class="w-full min-w-0 flex flex-col gap-2 sm:flex-row sm:items-center">
    <div class="min-w-0 flex flex-wrap items-center gap-2">
      {#if isInstalled}
        <PlayButton
          text="Play"
          icon="i-material-symbols-power-rounded"
          variant="success"
          wrapperClass="shrink-0"
          buttonClass="h-9 min-w-24 justify-center px-3"
          menuClass="h-9 w-9"
          on:play={handlePlayClick}
        />
      {:else}
        <Button
          text="パスを設定"
          leftIcon="i-material-symbols-folder-open-rounded"
          variant="warning"
          appendClass="h-9 justify-center px-3"
          wrappable
          on:click={() => (isOpenImportManually = true)}
        />
      {/if}
      <APopover let:close panelClass="right-0">
        <ButtonCancel
          icon="i-material-symbols-settings-rounded"
          iconClass="h-6 w-6"
          ariaLabel="その他の操作"
          slot="button"
        />
        <SettingPopover
          on:close={() => close(null)}
          on:selectChange={() => (isOpenImportManually = true)}
          on:selectDelete={() => (isOpenDelete = true)}
          on:selectOpen={() => {
            const path = element?.exePath ?? element?.lnkPath;
            if (path) commandOpenFolder(path);
          }}
          on:selectOtherInfomation={() => (isOpenOtherInformation = true)}
          on:selectShortcut={setAsShortcutGame}
        />
      </APopover>
      <ButtonCancel
        icon={isLike
          ? "i-material-symbols-favorite-rounded"
          : "i-material-symbols-favorite-outline-rounded"}
        colorClass={isLike
          ? "color-text-primary hover:color-text-primary"
          : "color-text-tertiary hover:color-text-primary"}
        ariaLabel={isLike ? "お気に入りを解除" : "お気に入りに追加"}
        on:click={toggleLike}
      />
      <ButtonCancel
        icon="i-material-symbols-qr-code"
        ariaLabel="QRコードを表示"
        on:click={() => (isOpenQrCode = true)}
      />
    </div>
  </div>

  <ImportManually
    bind:isOpen={isOpenImportManually}
    idInput={`${id}`}
    path={element.exePath ?? element.lnkPath}
    on:confirm={(e) => onChangeGame(e.detail)}
    on:cancel={() => (isOpenImportManually = false)}
  />
  <DeleteElement bind:isOpen={isOpenDelete} {element} />
  <OtherInformation bind:isOpen={isOpenOtherInformation} {element} />
  <QrCode bind:isOpen={isOpenQrCode} {id} {seiyaUrl} />
{/if}
