<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import PlayButton from "@/components/Work/PlayButton.svelte";
  import { push } from "svelte-spa-router";
  import {
    commandDeleteCollectionElement,
    commandGetCollectionElement,
    commandGetPlayTomeMinutes,
    commandOpenFolder,
    commandPlayGame,
    commandUpdateElementLike,
    commandUpdateElementPlayStatus,
    commandUpsertCollectionElement,
  } from "@/lib/command";
  import { showErrorToast } from "@/lib/toast";
  import { localStorageWritable } from "@/lib/utils";
  import ButtonIcon from "@/components/UI/ButtonIcon.svelte";
  import ButtonCancel from "@/components/UI/ButtonCancel.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import APopover from "@/components/UI/APopover.svelte";
  import SettingPopover from "@/components/Work/SettingPopover.svelte";
  import ImportManually from "@/components/Sidebar/ImportManually.svelte";
  import { deleteTab, tabs, selected } from "@/store/tabs";
  import DeleteElement from "@/components/Work/DeleteElement.svelte";
  import type { AllGameCacheOne, PlayStatus as PlayStatusType } from "@/lib/types";
  import { PlayStatus } from "@/lib/types";
  import OtherInformation from "@/components/Work/OtherInformation.svelte";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import QrCode from "@/components/Work/QRCode.svelte";
  import { startProcessMap } from "@/store/startProcessMap";
  import Select from "@/components/UI/Select.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";

  export let name: string;
  export let id: number;
  export let seiyaUrl: string;

  const isAdminRecord = localStorageWritable<Record<number, boolean>>(
    "play-admin-cache",
    {}
  );

  const play = async (isAdmin: boolean | undefined) => {
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
      showErrorToast(e as string);
    }
  };

  let isLike = false;
  let currentPlayStatus: PlayStatusType = PlayStatus.Unplayed;

  const toggleLike = async () => {
    await commandUpdateElementLike(id, !isLike);
    isLike = !isLike;
    sidebarCollectionElements.updateLike(id, isLike);
  };

  const handlePlayStatusSelect = (event: CustomEvent<{ value: string | number }>) => {
    const newStatus = event.detail.value as PlayStatusType;
    updatePlayStatus(newStatus);
  };

  const updatePlayStatus = async (newStatus: PlayStatusType) => {
    await commandUpdateElementPlayStatus(id, newStatus);
    currentPlayStatus = newStatus;
    sidebarCollectionElements.updatePlayStatus(id, newStatus);
  };

  const playStatusOptionsData: {
    label: string;
    value: PlayStatusType;
    icon: string;
    activeStyleClasses: string;
    activeIconTextColorClass: string; // アイコンとテキスト用の共通色クラス
  }[] = [
    {
      label: "未プレイ",
      value: PlayStatus.Unplayed,
      icon: "i-material-symbols-play-circle-outline-rounded",
      activeStyleClasses: "bg-gray-400 !hover:bg-gray-300 text-white border-gray-400",
      activeIconTextColorClass: "text-white"
    },
    {
      label: "プレイ中",
      value: PlayStatus.Playing,
      icon: "i-material-symbols-pause-circle-outline-rounded",
      activeStyleClasses: "bg-blue-500 !hover:bg-blue-400 text-white border-blue-500",
      activeIconTextColorClass: "text-white"
    },
    {
      label: "クリア済み",
      value: PlayStatus.Cleared,
      icon: "i-material-symbols-check-circle-outline-rounded",
      activeStyleClasses: "bg-green-700 !hover:bg-green-600 text-white border-green-700",
      activeIconTextColorClass: "text-white"
    },
  ];

  // Selectコンポーネントのoptionsプロパティ用 (labelとvalueのみ)
  $: selectOptionsForDropdown = playStatusOptionsData.map(opt => ({ label: opt.label, value: opt.value }));

  // 現在のプレイ状況に対応するスタイル情報を取得
  $: currentActiveStyleInfo = playStatusOptionsData.find(opt => opt.value === currentPlayStatus) || playStatusOptionsData[0];


  $: playTimePromise = commandGetPlayTomeMinutes(id);
  $: elementPromise = (async () => {
    const element = await commandGetCollectionElement(id);
    isLike = !!element.likeAt;
    currentPlayStatus = element.playStatus;
    return element;
  })();

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
    if (isChangedGameId) {
      deleteTab($tabs[$selected].id);
    }
    isOpenImportManually = false;
  };

  let isOpenDelete = false;
  let isOpenOtherInformation = false;
  let isOpenQrCode = false;
</script>

{#await elementPromise then element}
  <div class="flex items-center gap-4 flex-wrap w-full min-w-0">
    <PlayButton on:play={(e) => play(e.detail.isAdmin)} />
    <Button
      leftIcon="i-material-symbols-drive-file-rename-outline"
      text="Memo"
      on:click={() => push(`/memos/${id}?gamename=${name}`)}
    />
    <div class="flex items-center gap-2 ml-auto">
      <Select
        options={selectOptionsForDropdown}
        bind:value={currentPlayStatus}
        on:select={handlePlayStatusSelect}
        showSelectedCheck={true}
        title="プレイ状況を変更"
      >
        <ButtonBase
            variant={"normal"}
            appendClass={`h-8 px-3 flex items-center justify-between gap-1.5 min-w-32 text-sm transition-none ${currentActiveStyleInfo.activeStyleClasses}`}
            tooltip={{
              content: "プレイ状況: " + currentActiveStyleInfo.label,
              placement: "bottom",
              theme: "default",
              delay: 1000,
            }}
        >
            <div class="flex items-center gap-1 overflow-hidden">
                <div class="{currentActiveStyleInfo.icon} w-4 h-4 flex-shrink-0 {currentActiveStyleInfo.activeIconTextColorClass}" />
                <span class="text-xs font-medium truncate {currentActiveStyleInfo.activeIconTextColorClass}">{currentActiveStyleInfo.label}</span>
            </div>
            <div class="i-material-symbols-arrow-drop-down w-4 h-4 flex-shrink-0 {currentActiveStyleInfo.activeIconTextColorClass}" />
        </ButtonBase>
      </Select>
      <ButtonCancel
        icon="i-material-symbols-qr-code"
        on:click={() => (isOpenQrCode = true)}
      />
      <ButtonCancel
        icon={isLike
          ? "i-material-symbols-favorite-rounded color-accent-error"
          : "i-material-symbols-favorite-outline-rounded"}
        on:click={toggleLike}
      />
      <APopover let:close panelClass="right-0">
        <ButtonIcon icon="i-material-symbols-menu-rounded" slot="button" />
        <SettingPopover
          on:close={() => close(null)}
          on:selectChange={() => (isOpenImportManually = true)}
          on:selectDelete={() => (isOpenDelete = true)}
          on:selectOpen={() =>
            commandOpenFolder(element.exePath ?? element.lnkPath)}
          on:selectOtherInfomation={() => (isOpenOtherInformation = true)}
        />
      </APopover>
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
{/await}
