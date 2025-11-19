<script lang="ts">
  import { onMount } from "svelte";
  import { derived, writable } from "svelte/store";
  import { backgroundState } from "@/store/background";
  import type {
    CollectionElement,
    PlayStatus as PlayStatusType,
  } from "@/lib/types";
  import { PlayStatus } from "@/lib/types";
  import {
    commandGetAllElements,
    commandUpdateElementPlayStatus,
  } from "@/lib/command";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { showInfoToast, showErrorToast } from "@/lib/toast";
  import Button from "@/components/UI/Button.svelte";
  import IconButton from "@/components/UI/IconButton.svelte";
  import VirtualScroller from "@/components/UI/VirtualScroller.svelte";
  import MasonryLayoutForPlayStatus from "@/components/PlayStatusBulkEditor/MasonryLayoutForPlayStatus.svelte";
  import GameListLayout from "@/components/PlayStatusBulkEditor/GameListLayout.svelte";
  import { localStorageWritable } from "@/lib/utils";

  import { query as textQueryStore } from "@/store/query";
  import { currentSortOrder, currentAttributes } from "@/store/viewSettings";
  import { FILTER_BY_ATTRIBUTE } from "@/components/Sidebar/searchAttributes";
  import { sort as sortElementsOriginal } from "@/components/Sidebar/sort";
  import {
    collectionElementsToOptions as convertElementsToOptionsForFilter,
    type Option as FilterOption,
  } from "@/lib/filter";
  import { useFilter as useTextQueryFilter } from "@/lib/filter";

  let allGamesFromApi = writable<CollectionElement[]>([]);
  let selectedGameIdsStore = writable(new Set<number>());
  let targetPlayStatusStore = localStorageWritable<PlayStatusType>(
    "bulkEditorTargetPlayStatus",
    PlayStatus.Unplayed,
  );
  let isLoading = true;
  type ViewMode = "masonry" | "list";
  let viewModeStore = localStorageWritable<ViewMode>(
    "playStatusEditorViewMode",
    "masonry",
  );

  onMount(async () => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });
    isLoading = true;
    try {
      const games = await commandGetAllElements();
      allGamesFromApi.set(games);
    } catch (e) {
      showErrorToast("ゲームリストの読み込みに失敗しました。");
      console.error(e);
    } finally {
      isLoading = false;
    }
  });

  const textFilterOptionsForThisPage = derived(
    allGamesFromApi,
    ($allGames): FilterOption<number>[] =>
      convertElementsToOptionsForFilter($allGames),
  );

  const { filtered: textFilteredGameIdOptions } = useTextQueryFilter<number>(
    textQueryStore,
    textFilterOptionsForThisPage,
    () => convertElementsToOptionsForFilter($allGamesFromApi),
  );

  const processedDisplayGames = derived<
    [
      typeof allGamesFromApi,
      typeof textFilteredGameIdOptions,
      typeof currentAttributes,
      typeof currentSortOrder,
    ],
    CollectionElement[]
  >(
    [
      allGamesFromApi,
      textFilteredGameIdOptions,
      currentAttributes,
      currentSortOrder,
    ],
    ([$allGames, $textFilteredIdOpts, $attributeFilters, $sortOrder], set) => {
      const textFilteredIdSet = new Set(
        $textFilteredIdOpts.map((opt) => opt.value),
      );
      let filteredByText = $allGames.filter((game) =>
        textFilteredIdSet.has(game.id),
      );

      const activeAttributeFilters = $attributeFilters.filter(
        (attr) => attr.enabled,
      );
      let filteredByAttributes = filteredByText;
      if (activeAttributeFilters.length > 0) {
        filteredByAttributes = activeAttributeFilters.reduce(
          (acc, currentFilter) => {
            const filterFn = FILTER_BY_ATTRIBUTE[currentFilter.key];
            return filterFn ? filterFn(acc) : acc;
          },
          filteredByText,
        );
      }

      const sortedGroupedGames = sortElementsOriginal(
        filteredByAttributes,
        $sortOrder,
      );
      const sortedFlatGames = sortedGroupedGames.flatMap(
        (group) => group.elements,
      );

      set(sortedFlatGames);
    },
  );

  const displayGamesWithPreview = derived<
    [
      typeof processedDisplayGames,
      typeof selectedGameIdsStore,
      typeof targetPlayStatusStore,
    ],
    CollectionElement[]
  >(
    [processedDisplayGames, selectedGameIdsStore, targetPlayStatusStore],
    ([$processedGames, $selectedIds, $targetPlayStatus], set) => {
      const games = ($processedGames as CollectionElement[]).map(
        (game: CollectionElement) => {
          if ($selectedIds.has(game.id)) {
            return { ...game, playStatus: $targetPlayStatus };
          }
          return game;
        },
      );
      set(games);
    },
  );

  const toggleGameSelection = (gameId: number) => {
    selectedGameIdsStore.update((currentSet) => {
      const newSet = new Set(currentSet);
      if (newSet.has(gameId)) {
        newSet.delete(gameId);
      } else {
        newSet.add(gameId);
      }
      return newSet;
    });
  };

  const handleTargetPlayStatusChange = (newStatus: PlayStatusType) => {
    targetPlayStatusStore.set(newStatus);
  };

  const handleBulkUpdate = async () => {
    const currentSelectedIds = $selectedGameIdsStore;
    if (currentSelectedIds.size === 0) {
      showInfoToast("一括設定するゲームを選択してください。");
      return;
    }
    isLoading = true;
    let successCount = 0;

    const originalPlayStatusMap = new Map<number, PlayStatusType>();
    $allGamesFromApi.forEach((game) => {
      if (currentSelectedIds.has(game.id)) {
        originalPlayStatusMap.set(game.id, game.playStatus);
      }
    });

    const gameIdsToUpdate = Array.from(currentSelectedIds);
    const originalSelectedIdsForRollback = new Set(currentSelectedIds);

    try {
      for (const gameId of gameIdsToUpdate) {
        await commandUpdateElementPlayStatus(gameId, $targetPlayStatusStore);
        successCount++;
      }

      allGamesFromApi.update((currentGames) =>
        currentGames.map((game) =>
          originalSelectedIdsForRollback.has(game.id)
            ? { ...game, playStatus: $targetPlayStatusStore }
            : game,
        ),
      );
      selectedGameIdsStore.set(new Set());

      showInfoToast(`${successCount}件のゲームのプレイ状況を更新しました。`);
      await sidebarCollectionElements.refetch();
    } catch (e) {
      showErrorToast("プレイ状況の一括更新に失敗しました。");
      console.error(e);
      allGamesFromApi.update((currentGames) =>
        currentGames.map((game) => {
          if (originalPlayStatusMap.has(game.id)) {
            return { ...game, playStatus: originalPlayStatusMap.get(game.id)! };
          }
          return game;
        }),
      );
      selectedGameIdsStore.set(originalSelectedIdsForRollback);
    } finally {
      isLoading = false;
    }
  };

  const playStatusOptions: {
    label: string;
    value: PlayStatusType;
    icon?: string;
    activeStyleClasses: string; // 選択時のスタイル (ホバー含む)
    inactiveStyleClasses: string; // 非選択時のスタイル (ホバー含む)
  }[] = [
    {
      label: "未プレイ",
      value: PlayStatus.Unplayed,
      icon: "i-material-symbols-play-circle-outline-rounded",
      activeStyleClasses:
        "bg-gray-400 !hover:bg-gray-300 text-white border-gray-400",
      inactiveStyleClasses:
        "text-text-primary bg-bg-button hover:bg-bg-button-hover border-border-primary",
    },
    {
      label: "プレイ中",
      value: PlayStatus.Playing,
      icon: "i-material-symbols-pause-circle-outline-rounded",
      activeStyleClasses:
        "bg-blue-500 !hover:bg-blue-400 text-white border-blue-500",
      inactiveStyleClasses:
        "text-text-primary bg-bg-button hover:bg-bg-button-hover border-border-primary",
    },
    {
      label: "クリア済み",
      value: PlayStatus.Cleared,
      icon: "i-material-symbols-check-circle-outline-rounded",
      activeStyleClasses:
        "bg-green-700 !hover:bg-green-600 text-white border-green-700",
      inactiveStyleClasses:
        "text-text-primary bg-bg-button hover:bg-bg-button-hover border-border-primary",
    },
  ];
</script>

<div class="p-4 md:p-6 h-full flex flex-col gap-4">
  <div
    class="flex flex-wrap items-center justify-between gap-4 p-3 md:p-4 rounded-lg bg-bg-secondary sticky top-0 z-10 shadow"
  >
    <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
      <div class="text-(body text-primary) font-medium whitespace-nowrap">
        目標の状態:
      </div>
      <div class="flex gap-2 flex-wrap">
        {#each playStatusOptions as option (option.value)}
          <Button
            text={option.label}
            leftIcon={option.icon}
            variant={"normal"}
            on:click={() => handleTargetPlayStatusChange(option.value)}
            appendClass={`px-3 py-1 text-sm ${$targetPlayStatusStore === option.value ? option.activeStyleClasses : option.inactiveStyleClasses}`}
          />
        {/each}
      </div>
    </div>
    <div class="flex items-center gap-2">
      <IconButton
        icon="i-material-symbols-view-module-outline-rounded"
        on:click={() => viewModeStore.set("masonry")}
        tooltip={{ content: "タイル表示" }}
        appendClass={$viewModeStore === "masonry" ? "!bg-[#C0C0C0]" : ""}
      />
      <IconButton
        icon="i-material-symbols-view-list-outline-rounded"
        on:click={() => viewModeStore.set("list")}
        tooltip={{ content: "リスト表示" }}
        appendClass={$viewModeStore === "list" ? "!bg-[#C0C0C0]" : ""}
      />
      <Button
        text={`選択中 (${$selectedGameIdsStore.size}) を設定`}
        leftIcon="i-material-symbols-library-add-check-outline-rounded"
        variant="accent"
        disabled={$selectedGameIdsStore.size === 0 || isLoading}
        on:click={handleBulkUpdate}
        appendClass="px-3 py-1 text-sm whitespace-nowrap"
      />
    </div>
  </div>

  {#if isLoading && $allGamesFromApi.length === 0}
    <div class="flex-1 flex items-center justify-center text-(lg text-primary)">
      読み込み中...
    </div>
  {:else if $allGamesFromApi.length === 0}
    <div class="flex-1 flex items-center justify-center text-(lg text-primary)">
      登録されているゲームがありません。
    </div>
  {:else}
    <div class="flex-1 min-h-0 overflow-hidden">
      {#if $viewModeStore === "masonry"}
        <VirtualScroller
          className="p-4"
          let:setVirtualHeight
          let:contentsWidth
          let:contentsScrollY
          let:containerHeight
          let:contentsScrollTo
        >
          <MasonryLayoutForPlayStatus
            elementsStore={displayGamesWithPreview}
            selectedIdsStore={selectedGameIdsStore}
            previewTargetPlayStatus={$targetPlayStatusStore}
            onToggleSelection={(id) => toggleGameSelection(id)}
            {setVirtualHeight}
            {contentsScrollY}
            {contentsWidth}
            {containerHeight}
            {contentsScrollTo}
            minItemWidth={216}
            itemGap={12}
            titleAreaHeight={70}
            placeholderAspectRatio={3 / 4}
            tileInternalPadding={8}
          />
        </VirtualScroller>
      {:else if $viewModeStore === "list"}
        <GameListLayout
          elementsStore={displayGamesWithPreview}
          selectedIdsStore={selectedGameIdsStore}
          previewTargetPlayStatus={$targetPlayStatusStore}
          onToggleSelection={toggleGameSelection}
          itemHeight={72}
        />
      {/if}
    </div>
  {/if}
</div>
