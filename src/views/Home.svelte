<script lang="ts">
  import {
    commandGetCollectionElement,
    commandUpdateAllGameCache,
    commandUpdateCollectionElementThumbnails,
  } from "@/lib/command";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Icon from "/icon.png";
  import { link } from "svelte-spa-router";
  import LinkText from "@/components/UI/LinkText.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import VirtualScroller from "@/components/UI/VirtualScroller.svelte";
  import VirtualScrollerMasonry from "@/components/UI/VirtualScrollerMasonry.svelte";
  import { derived } from "svelte/store";
  import Button from "@/components/UI/Button.svelte";
  import { scrapeAllGameCacheOnes } from "@/lib/scrapeAllGame";
  import { showErrorToast, showInfoToast } from "@/lib/toast";
  import RecentlyPlayedScroller from "@/components/Home/RecentlyPlayedScroller.svelte";
  import ZappingGameItem from "@/components/Home/ZappingGameItem.svelte";
  import { formatLastPlayed } from "@/lib/utils";
  import Card from "@/components/UI/Card.svelte";
  import type { SvelteComponent } from "svelte";
  import ArrowButton from "@/components/Home/ArrowButton.svelte";
  import { onMount, tick } from "svelte";
  import { backgroundState } from "@/store/background";

  let scrollable: RecentlyPlayedScroller;

  const reInitScroller = () => {
    if (!scrollable) return;
    scrollable.reInit();
  };

  onMount(() => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });
  });

  $: if ($recentlyPlayed && scrollable) {
    tick().then(reInitScroller);
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
</script>

<svelte:window on:resize={reInitScroller} />

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

    <!-- Hero Section & Recently Played -->
    {#if $loading}
      <div class="w-full h-64 rounded-2xl glass animate-pulse" />
    {:else if $recentlyPlayed.length > 0}
      {@const heroGame = $recentlyPlayed[0]}
      <!-- Hero Section -->
      <div
        class="relative w-full h-80 rounded-2xl overflow-hidden group shadow-2xl mb-8"
      >
        <!-- Background Image -->
        {#if heroGame.thumbnail}
          <img
            src={convertFileSrc(heroGame.thumbnail)}
            alt={heroGame.gamename}
            class="absolute inset-0 w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
          />
          <div
            class="absolute inset-0 bg-gradient-to-t from-bg-primary via-bg-primary/50 to-transparent"
          />
        {/if}

        <!-- Content -->
        <div
          class="absolute bottom-0 left-0 p-8 w-full flex flex-col gap-4 items-start"
        >
          <div class="space-y-1">
            <div
              class="text-accent-accent font-bold tracking-wider text-sm uppercase"
            >
              Recently Played
            </div>
            <h2
              class="text-4xl font-bold text-white drop-shadow-lg leading-tight max-w-3xl"
            >
              {heroGame.gamename}
            </h2>
          </div>

          <div class="flex items-center gap-4 mt-2">
            <a
              href="/memos/{heroGame.id}?gamename={heroGame.gamename}"
              use:link
            >
              <button
                class="btn-primary px-8 py-3 rounded-full font-bold flex items-center gap-2"
              >
                <div class="i-material-symbols-play-arrow-rounded text-xl" />
                <span>Play Now</span>
              </button>
            </a>
            <div
              class="glass px-4 py-2 rounded-full text-sm text-white/80 flex items-center gap-2"
            >
              <div class="i-material-symbols-history-rounded" />
              <span>{formatLastPlayed(heroGame.lastPlayAt)}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Other Recently Played -->
      {#if $recentlyPlayed.length > 1}
        <div class="space-y-4 mb-8">
          <div class="flex items-center justify-between">
            <h3 class="text-(text-primary h3) font-medium">Continue Playing</h3>
            <div class="flex items-center gap-2">
              <ArrowButton back on:click={() => scrollable.scrollPrev()} />
              <ArrowButton on:click={() => scrollable.scrollNext()} />
            </div>
          </div>
          <div class="relative">
            <RecentlyPlayedScroller bind:this={scrollable}>
              {#each $recentlyPlayed.slice(1) as element (element.id)}
                {@const isPortrait =
                  element.thumbnailHeight &&
                  element.thumbnailWidth &&
                  element.thumbnailHeight > element.thumbnailWidth}
                {@const ar = isPortrait ? 4 / 5 : 5 / 4}
                {@const heightRem = 12}
                {@const widthRem = heightRem * ar}
                <div
                  class="embla__slide flex-shrink-0"
                  style="flex: 0 0 {widthRem}rem; height: {heightRem}rem; padding-left: 1rem;"
                >
                  <ZappingGameItem
                    collectionElement={element}
                    objectFit="cover"
                    objectPosition="top"
                  >
                    <div
                      slot="info"
                      class="text-xs text-text-tertiary px-1 truncate mb-1"
                    >
                      {formatLastPlayed(element.lastPlayAt)}
                    </div>
                  </ZappingGameItem>
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
