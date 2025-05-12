<script lang="ts">
  import { onMount } from "svelte";
  import { derived, writable } from "svelte/store"; // writable もインポート
  import type { CollectionElement, PlayStatus as PlayStatusType } from "@/lib/types";
  import { PlayStatus } from "@/lib/types";
  import { commandGetAllElements, commandUpdateElementPlayStatus } from "@/lib/command";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { showInfoToast, showErrorToast } from "@/lib/toast";
  import Button from "@/components/UI/Button.svelte";
  import VirtualScroller from "@/components/UI/VirtualScroller.svelte";
  import MasonryLayoutForPlayStatus from "@/components/PlayStatusBulkEditor/MasonryLayoutForPlayStatus.svelte";

  import { query as textQueryStore } from "@/store/query";
  import { currentSortOrder, currentAttributes } from "@/store/viewSettings";
  import { FILTER_BY_ATTRIBUTE, type Attribute } from "@/components/Sidebar/searchAttributes";
  import { sort as sortElementsOriginal, type SortOrder } from "@/components/Sidebar/sort";
  import { collectionElementsToOptions as convertElementsToOptionsForFilter, type Option as FilterOption } from "@/lib/filter";
  import { useFilter as useTextQueryFilter } from "@/lib/filter";


  let allGamesFromApi = writable<CollectionElement[]>([]);
  let selectedGameIdsStore = writable(new Set<number>());
  let targetPlayStatusStore = writable<PlayStatusType>(PlayStatus.Unplayed);
  let isLoading = true;

  onMount(async () => {
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

  const textFilterOptionsForThisPage = derived(allGamesFromApi, ($allGames): FilterOption<number>[] => // ★戻り値の型を明示
    convertElementsToOptionsForFilter($allGames)
  );

  const { filtered: textFilteredGameIdOptions } = useTextQueryFilter<number>(
    textQueryStore,
    textFilterOptionsForThisPage,
    () => convertElementsToOptionsForFilter($allGamesFromApi)
  );

  const processedDisplayGames = derived< // ★ derivedの型引数を追加
    [typeof allGamesFromApi, typeof textFilteredGameIdOptions, typeof currentAttributes, typeof currentSortOrder],
    CollectionElement[] // ★ このストアが持つデータの型
  >(
    [allGamesFromApi, textFilteredGameIdOptions, currentAttributes, currentSortOrder],
    ([$allGames, $textFilteredIdOpts, $attributeFilters, $sortOrder], set) => {
      const textFilteredIdSet = new Set($textFilteredIdOpts.map(opt => opt.value));
      let filteredByText = $allGames.filter(game => textFilteredIdSet.has(game.id));

      const activeAttributeFilters = $attributeFilters.filter(attr => attr.enabled);
      let filteredByAttributes = filteredByText;
      if (activeAttributeFilters.length > 0) {
        filteredByAttributes = activeAttributeFilters.reduce((acc, currentFilter) => {
          const filterFn = FILTER_BY_ATTRIBUTE[currentFilter.key];
          return filterFn ? filterFn(acc) : acc;
        }, filteredByText);
      }

      const sortedGroupedGames = sortElementsOriginal(filteredByAttributes, $sortOrder);
      const sortedFlatGames = sortedGroupedGames.flatMap(group => group.elements);

      set(sortedFlatGames);
    }
  );

  const displayGamesWithPreview = derived< // ★ derivedの型引数を追加
    [typeof processedDisplayGames, typeof selectedGameIdsStore, typeof targetPlayStatusStore],
    CollectionElement[] // ★ このストアが持つデータの型
  >(
    [processedDisplayGames, selectedGameIdsStore, targetPlayStatusStore],
    ([$processedGames, $selectedIds, $targetPlayStatus], set) => {
        // $processedGames が CollectionElement[] であることを TypeScript に伝える
        const games = ($processedGames as CollectionElement[]).map((game: CollectionElement) => { // ★ game に型注釈
            if($selectedIds.has(game.id)) {
                return {...game, playStatus: $targetPlayStatus };
            }
            return game;
        });
        set(games);
    }
  );


  const toggleGameSelection = (gameId: number) => {
    selectedGameIdsStore.update(currentSet => {
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
    $allGamesFromApi.forEach(game => {
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

      allGamesFromApi.update(currentGames =>
        currentGames.map(game =>
          originalSelectedIdsForRollback.has(game.id) ? { ...game, playStatus: $targetPlayStatusStore } : game
        )
      );
      selectedGameIdsStore.set(new Set());

      showInfoToast(`${successCount}件のゲームのプレイ状況を更新しました。`);
      await sidebarCollectionElements.refetch();

    } catch (e) {
      showErrorToast("プレイ状況の一括更新に失敗しました。");
      console.error(e);
      allGamesFromApi.update(currentGames =>
        currentGames.map(game => {
          if (originalPlayStatusMap.has(game.id)) {
            return { ...game, playStatus: originalPlayStatusMap.get(game.id)! };
          }
          return game;
        })
      );
      selectedGameIdsStore.set(originalSelectedIdsForRollback);
    } finally {
      isLoading = false;
    }
  };

  const playStatusOptions: { label: string; value: PlayStatusType; icon?: string }[] = [
    { label: "未プレイ", value: PlayStatus.Unplayed, icon: "i-material-symbols-play-circle-outline-rounded" },
    { label: "プレイ中", value: PlayStatus.Playing, icon: "i-material-symbols-pause-circle-outline-rounded" },
    { label: "クリア済み", value: PlayStatus.Cleared, icon: "i-material-symbols-check-circle-outline-rounded" },
  ];

</script>

<div class="p-4 md:p-6 h-full flex flex-col gap-4">
  <div class="flex flex-wrap items-center justify-between gap-4 p-3 md:p-4 rounded-lg bg-bg-secondary sticky top-0 z-10 shadow">
    <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
      <div class="text-(body text-primary) font-medium whitespace-nowrap">目標の状態:</div>
      <div class="flex gap-2 flex-wrap">
        {#each playStatusOptions as option (option.value)}
          <Button
            text={option.label}
            leftIcon={option.icon}
            variant={$targetPlayStatusStore === option.value ? "success" : "normal"}
            on:click={() => handleTargetPlayStatusChange(option.value)}
            appendClass="px-3 py-1 text-sm"
          />
        {/each}
      </div>
    </div>
    <Button
      text={`選択中 (${$selectedGameIdsStore.size}) を設定`}
      leftIcon="i-material-symbols-library-add-check-outline-rounded"
      variant="accent"
      disabled={$selectedGameIdsStore.size === 0 || isLoading}
      on:click={handleBulkUpdate}
      appendClass="px-3 py-1 text-sm whitespace-nowrap"
    />
  </div>

  {#if isLoading && $allGamesFromApi.length === 0}
    <div class="flex-1 flex items-center justify-center text-(lg text-primary)">読み込み中...</div>
  {:else if $allGamesFromApi.length === 0}
    <div class="flex-1 flex items-center justify-center text-(lg text-primary)">登録されているゲームがありません。</div>
  {:else}
    <div class="flex-1 min-h-0 overflow-hidden">
      <VirtualScroller
        className="p-1"
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
          placeholderAspectRatio={3/4}
          tileInternalPadding={8}
        />
      </VirtualScroller>
    </div>
  {/if}
</div>
