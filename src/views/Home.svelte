<script lang="ts">
  import {
    commandGetCollectionElement,
    commandUpdateAllGameCache,
    commandUpdateCollectionElementThumbnails,
    commandPlayGame,
  } from "@/lib/command";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Icon from "/icon.png";
  import { link, push } from "svelte-spa-router";
  import LinkText from "@/components/UI/LinkText.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import VirtualScroller from "@/components/UI/VirtualScroller.svelte";
  import VirtualScrollerMasonry from "@/components/UI/VirtualScrollerMasonry.svelte";
  import { derived } from "svelte/store";
  import Button from "@/components/UI/Button.svelte";
  import { scrapeAllGameCacheOnes } from "@/lib/scrapeAllGame";
  import { showErrorToast, showInfoToast } from "@/lib/toast";
  import RecentlyPlayedScroller from "@/components/Home/RecentlyPlayedScroller.svelte";
  import GameCard from "@/components/UI/GameCard.svelte";
  import { formatLastPlayed, localStorageWritable } from "@/lib/utils";
  import Card from "@/components/UI/Card.svelte";
  import type { SvelteComponent } from "svelte";
  import ArrowButton from "@/components/Home/ArrowButton.svelte";
  import { onMount, tick } from "svelte";
  import { backgroundState } from "@/store/background";
  import { startProcessMap } from "@/store/startProcessMap";

  let scrollable: RecentlyPlayedScroller;

  onMount(() => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });
  });

  $: if ($recentlyPlayed && scrollable) {
    tick().then(() => {
      scrollable.reInit();
    });
  }

  import Skeleton from "@/components/UI/Skeleton.svelte";

  const memoRegex = /^smde_memo-(\d+)$/;
  const memoPromises = Promise.all(
    Object.keys(localStorage)
      .map((v) => +(v.match(memoRegex)?.[1] ?? "0"))
      .filter((v) => v)
      .map((v) => commandGetCollectionElement(v)),
  );

  let isOpenGettingStarted = true;

  const loading = sidebarCollectionElements.loading;
  const shown = sidebarCollectionElements.shown;
  const flattenShown = derived(shown, ($shown) =>
    $shown.flatMap((v) => v.elements),
  );

  const recentlyPlayed = derived(flattenShown, ($flattenShown) =>
    $flattenShown
      .filter((v) => v.lastPlayAt)
      .sort((a, b) => {
        const dateA = new Date(a.lastPlayAt!);
        const dateB = new Date(b.lastPlayAt!);
        return dateB.getTime() - dateA.getTime();
      })
      .slice(0, 10),
  );

  let disabledRefetchThumbnail = false;
  const refetchThumbnail = async () => {
    try {
      disabledRefetchThumbnail = true;
      const ids = $flattenShown
        .filter((v) => !v.thumbnailWidth && !v.thumbnailHeight)
        .map((v) => v.id);
      const caches = await scrapeAllGameCacheOnes(ids);
      await commandUpdateAllGameCache(caches);
      await commandUpdateCollectionElementThumbnails(ids);
      await sidebarCollectionElements.refetch();
      showInfoToast("サムネイルの再取得が完了しました");
    } catch (e) {
      showErrorToast("サムネイルの再取得に失敗しました");
      console.error(e);
    } finally {
      disabledRefetchThumbnail = false;
    }
  };

  let innerWidth = 0;

  const isAdminRecord = localStorageWritable<Record<number, boolean>>(
    "play-admin-cache",
    {},
  );

  const handlePlay = async (id: number) => {
    let _isAdmin = false;
    const cache = $isAdminRecord[id];
    if (cache) {
      _isAdmin = cache;
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
</script>

<svelte:window bind:innerWidth />

<VirtualScroller
  className="p-8"
  let:containerHeight
  let:contentsWidth
  let:contentsScrollY
  let:setVirtualHeight
  let:contentsScrollTo
>
  <div class="space-y-8 mb-2" slot="header">
    <div class="flex items-center gap-2 w-full">
      <img src={Icon} alt="launcherg icon" class="h-12" />
      <div class="font-logo text-(8 text-primary)">Launcherg</div>
    </div>

    <div class="grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-6">
      {#if $loading}
        <div class="p-6 rounded-xl glass space-y-3">
          <Skeleton width="40%" height="1.5rem" />
          <div class="space-y-2">
            <Skeleton width="100%" height="1rem" />
            <Skeleton width="80%" height="1rem" />
          </div>
        </div>
        <div class="p-6 rounded-xl glass space-y-3">
          <Skeleton width="30%" height="1.5rem" />
          <div class="space-y-2">
            <Skeleton width="60%" height="1rem" />
            <Skeleton width="50%" height="1rem" />
          </div>
        </div>
      {:else}
        {#if $sidebarCollectionElements.length === 0 && isOpenGettingStarted}
          <Card title="Getting started">
            持っているゲームをこのランチャーに登録してみましょう。左のサイドバーにある「Add」ボタンから自動で追加できます。
          </Card>
        {/if}

        <Card title="Help">
          <div class="flex-(~ col) gap-2">
            <LinkText
              href="https://youtu.be/GCTj6eRRgAM?si=WRFuBgNErwTJsNnk"
              text="1分でわかる Launcherg"
            />
            <LinkText
              href="https://ryoha000.hatenablog.com/entry/2023/09/24/003605"
              text="よくある Q&A"
            />
          </div>
        </Card>

        <Card title="Memo">
          {#await memoPromises then elements}
            {#if elements.length === 0 && $sidebarCollectionElements.length !== 0}
              <div class="text-(text-tertiary body)">
                このアプリにはメモ機能があります。サイドバーからゲームを選択して「Memo」ボタンを押すことでそのゲームについてメモを取ることができます。
              </div>
            {:else if elements.length === 0}
              <div class="text-(text-tertiary body)">
                メモはまだありません。
              </div>
            {:else}
              <div class="gap-1 flex-(~ col)">
                {#each elements as element (element.id)}
                  <a
                    use:link
                    href="/memos/{element.id}?gamename={element.gamename}"
                    class="text-(text-link body2) hover:underline-(1px text-link)"
                  >
                    メモ - {element.gamename}
                  </a>
                {/each}
              </div>
            {/if}
          {/await}
        </Card>
      {/if}
    </div>

    <!-- Recently Played Section -->
    {#if $loading}
      <div class="space-y-2">
        <div class="flex items-center">
          <Skeleton width="150px" height="1.5rem" />
        </div>
        <div class="flex gap-4 overflow-hidden">
          {#each Array(5) as _}
            <div class="flex-shrink-0" style="width: 10rem;">
              <div class="aspect-[4/5] rounded-lg overflow-hidden mb-1">
                <Skeleton width="100%" height="100%" />
              </div>
              <Skeleton width="80%" height="0.8rem" />
            </div>
          {/each}
        </div>
      </div>
    {:else if $recentlyPlayed.length > 0}
      {@const mostRecent = $recentlyPlayed[0]}
      {@const recentHistory = $recentlyPlayed.slice(1)}
      {@const TARGET_AREA =
        innerWidth < 768 ? 50000 : innerWidth < 1280 ? 70000 : 100000}
      {@const imageWidth = (() => {
        if (mostRecent.thumbnailWidth && mostRecent.thumbnailHeight) {
          const ratio = mostRecent.thumbnailWidth / mostRecent.thumbnailHeight;
          return Math.sqrt(TARGET_AREA * ratio);
        }
        return 192; // Default width (w-48)
      })()}

      <!-- Hero Section for Most Recent Game -->
      <div
        role="button"
        tabindex="0"
        on:click={() =>
          push(`/works/${mostRecent.id}?gamename=${mostRecent.gamename}`)}
        on:keydown={(e) =>
          e.key === "Enter" &&
          push(`/works/${mostRecent.id}?gamename=${mostRecent.gamename}`)}
        class="relative w-full h-[400px] rounded-2xl overflow-hidden group mb-8 block cursor-pointer"
      >
        <!-- Background Image -->
        <div class="absolute inset-0 z-0">
          {#if mostRecent.thumbnail}
            <img
              src={`${convertFileSrc(mostRecent.thumbnail)}?v=${mostRecent.updatedAt}`}
              alt={mostRecent.gamename}
              class="w-full h-full object-cover opacity-60 blur-md scale-105 transition-transform duration-700 group-hover:scale-110"
            />
          {:else}
            <div class="w-full h-full bg-bg-secondary/50" />
          {/if}
          <div
            class="absolute inset-0 bg-gradient-to-t from-bg-primary via-bg-primary/50 to-transparent"
          />
          <div
            class="absolute inset-0 bg-gradient-to-r from-bg-primary/80 via-transparent to-transparent"
          />
        </div>

        <!-- Content -->
        <div class="relative z-10 h-full flex items-end p-8 gap-8">
          <!-- Cover Art -->

          <div
            class="shrink-0 rounded-lg overflow-hidden shadow-2xl transform transition-transform duration-300 group-hover:-translate-y-2"
            style="width: {imageWidth}px;"
          >
            <img
              src={mostRecent.thumbnail
                ? `${convertFileSrc(mostRecent.thumbnail)}?v=${mostRecent.updatedAt}`
                : ""}
              alt="Cover"
              class="w-full h-auto"
            />
          </div>

          <!-- Info -->
          <div class="flex-1 mb-2">
            <div class="flex items-center gap-3 mb-2">
              <span class="text-text-tertiary text-sm">
                Last played: {formatLastPlayed(mostRecent.lastPlayAt)}
              </span>
            </div>

            <h2
              class="text-4xl font-bold text-white mb-6 drop-shadow-lg line-clamp-2"
            >
              {mostRecent.gamename}
            </h2>

            <div class="flex gap-4">
              <Button
                text="Play Now"
                leftIcon="i-material-symbols-play-arrow-rounded"
                variant="accent"
                appendClass="!px-8 !py-3 !text-lg shadow-lg shadow-accent-accent/20"
                on:click={(e) => {
                  e.stopPropagation();
                  handlePlay(mostRecent.id);
                }}
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Recent History Scroller -->
      {#if recentHistory.length > 0}
        <div class="space-y-2">
          <div class="flex items-center">
            <h3 class="text-(text-primary h3) font-medium mr-auto">
              最近の履歴
            </h3>
            <div class="flex items-center">
              <ArrowButton back on:click={() => scrollable.scrollPrev()} />
              <ArrowButton on:click={() => scrollable.scrollNext()} />
            </div>
          </div>
          <div class="relative">
            <RecentlyPlayedScroller bind:this={scrollable}>
              {#each recentHistory as element (element.id)}
                {@const isPortrait =
                  element.thumbnailHeight &&
                  element.thumbnailWidth &&
                  element.thumbnailHeight > element.thumbnailWidth}
                {@const ar = isPortrait ? 4 / 5 : 5 / 4}
                {@const heightRem = 13}
                {@const widthRem = heightRem * ar}
                <div
                  class="embla__slide flex-shrink-0"
                  style="flex: 0 0 {widthRem}rem; padding-left: 1rem;"
                >
                  <div class="text-sm text-text-tertiary px-1 truncate mb-1">
                    {formatLastPlayed(element.lastPlayAt)}
                  </div>
                  <div style="height: {heightRem}rem;" class="w-full">
                    <GameCard collectionElement={element} />
                  </div>
                </div>
              {/each}
            </RecentlyPlayedScroller>
          </div>
        </div>
      {/if}
    {/if}

    <div class="flex items-center gap-4 flex-wrap">
      <h3 class="text-(text-primary h3) font-medium mr-auto">登録したゲーム</h3>
      <a href="/settings/play-status" use:link class="ml-auto md:ml-0">
        <Button
          text="プレイ状況を一括編集"
          leftIcon="i-material-symbols-label-outline-rounded"
          variant="normal"
          tooltip={{
            content: "すべてのゲームのプレイ状況を一括で設定します",
            placement: "bottom",
            theme: "default",
            delay: 1000,
          }}
        />
      </a>
      <Button
        leftIcon="i-material-symbols-refresh-rounded"
        text="サムネイルを再取得する"
        disabled={disabledRefetchThumbnail}
        on:click={refetchThumbnail}
      />
    </div>
  </div>

  {#if $loading}
    <div class="p-8 grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-4">
      {#each Array(12) as _}
        <div class="space-y-2">
          <div class="aspect-[4/5] rounded-lg overflow-hidden">
            <Skeleton width="100%" height="100%" />
          </div>
          <Skeleton width="90%" height="1rem" />
        </div>
      {/each}
    </div>
  {:else}
    <VirtualScrollerMasonry
      elements={flattenShown}
      {setVirtualHeight}
      {contentsScrollY}
      {contentsWidth}
      {containerHeight}
      {contentsScrollTo}
    />
  {/if}
</VirtualScroller>
