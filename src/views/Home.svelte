<script lang="ts">
  import {
    commandGetCollectionElement,
    commandUpdateAllGameCache,
    commandUpdateCollectionElementThumbnails,
  } from "@/lib/command";
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
</script>

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
      <div class="space-y-2">
        <div class="flex items-center">
          <h3 class="text-(text-primary h3) font-medium mr-auto">最近プレイ</h3>
          <div class="flex items-center">
            <ArrowButton back on:click={() => scrollable.scrollPrev()} />
            <ArrowButton on:click={() => scrollable.scrollNext()} />
          </div>
        </div>
        <div class="relative">
          <RecentlyPlayedScroller bind:this={scrollable}>
            {#each $recentlyPlayed as element (element.id)}
              {@const isPortrait =
                element.thumbnailHeight &&
                element.thumbnailWidth &&
                element.thumbnailHeight > element.thumbnailWidth}
              {@const ar = isPortrait ? 4 / 5 : 5 / 4}
              {@const heightRem = 13}
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
                    class="text-sm text-text-tertiary px-1 truncate mb-1"
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
