<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import PlayHeatmap from "@/components/Work/PlayHeatmap.svelte";
  import Skeleton from "@/components/UI/Skeleton.svelte";
  import { commandGetCollectionElementDailyPlayTimes } from "@/lib/command";
  import {
    buildPlayHeatmap,
    formatLocalDateKey,
    parseLocalDateKey,
  } from "@/lib/playHeatmap";
  import { mergeDailyPlayTimes } from "@/lib/playHeatmapHelper";
  import { playStatusIcon, playStatusLabel } from "@/lib/playStatus";
  import type {
    CollectionElement,
    CollectionElementDailyPlayTime,
    PlayStatus as PlayStatusType,
  } from "@/lib/types";
  import { PlayStatus } from "@/lib/types";
  import { formatLastPlayed, formatPlayTime } from "@/lib/utils";
  import { backgroundState } from "@/store/background";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";

  type OverviewCard = {
    label: string;
    value: string;
    caption: string;
    icon: string;
    toneClass: string;
  };

  type StatusStat = {
    status: PlayStatusType;
    label: string;
    icon: string;
    count: number;
    percent: number;
    barClass: string;
    softClass: string;
  };

  type BarStat = {
    label: string;
    value: number;
    valueText: string;
    percent: number;
  };

  type StreakStats = {
    longest: number;
    recent: number;
    latestDate: string | null;
  };

  const totalHeatmapColor = { r: 72, g: 122, b: 249 };
  const now = new Date();
  const statusOrder: PlayStatusType[] = [
    PlayStatus.Unplayed,
    PlayStatus.Playing,
    PlayStatus.Cleared,
    PlayStatus.Interrupted,
  ];

  const statusStyles: Record<
    PlayStatusType,
    { barClass: string; softClass: string }
  > = {
    [PlayStatus.Unplayed]: {
      barClass: "bg-ui-tertiary",
      softClass: "bg-ui-tertiary/12 text-text-tertiary",
    },
    [PlayStatus.Playing]: {
      barClass: "bg-accent-primary",
      softClass: "bg-accent-primary/12 text-accent-primary",
    },
    [PlayStatus.Cleared]: {
      barClass: "bg-accent-success",
      softClass: "bg-accent-success/12 text-accent-success",
    },
    [PlayStatus.Interrupted]: {
      barClass: "bg-accent-warning",
      softClass: "bg-accent-warning/12 text-accent-warning",
    },
    [PlayStatus.LegacyShelved]: {
      barClass: "bg-accent-warning",
      softClass: "bg-accent-warning/12 text-accent-warning",
    },
  };

  const weekdayLabels = ["日", "月", "火", "水", "木", "金", "土"];

  let dailyPlayTimes: CollectionElementDailyPlayTime[] = [];
  let totalPlayTimes: CollectionElementDailyPlayTime[] = [];
  let isDailyStatsLoading = true;
  let hasDailyStatsError = false;
  let dailyStatsRequestId = 0;
  const loading = sidebarCollectionElements.loading;

  const parseDateTime = (value: string | null | undefined) => {
    if (!value) return null;
    const normalized = value.includes("T") ? value : value.replace(" ", "T");
    const date = new Date(normalized);
    return Number.isNaN(date.getTime()) ? null : date;
  };

  const sortByNullableDateDesc = (
    a: string | null | undefined,
    b: string | null | undefined,
  ) => {
    return (
      (parseDateTime(b)?.getTime() ?? 0) - (parseDateTime(a)?.getTime() ?? 0)
    );
  };

  const createPercent = (value: number, max: number) =>
    max > 0 ? Math.round((value / max) * 100) : 0;

  const formatDateLabel = (dateKey: string | null) => {
    if (!dateKey) return "記録なし";
    const date = parseLocalDateKey(dateKey);
    return date
      ? date.toLocaleDateString("ja-JP", {
          month: "numeric",
          day: "numeric",
          weekday: "short",
        })
      : dateKey;
  };

  const addDays = (date: Date, days: number) => {
    const next = new Date(date);
    next.setDate(next.getDate() + days);
    return next;
  };

  const calculateStreakStats = (
    rows: CollectionElementDailyPlayTime[],
  ): StreakStats => {
    const activeDates = Array.from(
      new Set(
        rows
          .filter((row) => row.playTimeSeconds > 0)
          .map((row) => row.playDate.slice(0, 10)),
      ),
    ).sort();

    let longest = 0;
    let currentRun = 0;
    let previousDate: Date | null = null;

    for (const dateKey of activeDates) {
      const date = parseLocalDateKey(dateKey);
      if (!date) continue;

      const isConsecutive =
        previousDate !== null &&
        formatLocalDateKey(addDays(previousDate, 1)) === dateKey;
      currentRun = isConsecutive ? currentRun + 1 : 1;
      longest = Math.max(longest, currentRun);
      previousDate = date;
    }

    const latestDate = activeDates[activeDates.length - 1] ?? null;
    let recent = 0;
    if (latestDate) {
      let cursor = parseLocalDateKey(latestDate);
      while (cursor && activeDates.includes(formatLocalDateKey(cursor))) {
        recent += 1;
        cursor = addDays(cursor, -1);
      }
    }

    return { longest, recent, latestDate };
  };

  const buildWeekdayStats = (
    rows: CollectionElementDailyPlayTime[],
  ): BarStat[] => {
    const totals = new Array(7).fill(0) as number[];
    for (const row of rows) {
      const date = parseLocalDateKey(row.playDate);
      if (!date || row.playTimeSeconds <= 0) continue;
      totals[date.getDay()] += row.playTimeSeconds;
    }

    const max = Math.max(...totals, 0);
    return weekdayLabels.map((label, index) => ({
      label,
      value: totals[index],
      valueText: formatPlayTime(totals[index]),
      percent: createPercent(totals[index], max),
    }));
  };

  const buildBucketStats = (labels: string[], limit = 6): BarStat[] => {
    const counts = new Map<string, number>();
    for (const label of labels) {
      counts.set(label, (counts.get(label) ?? 0) + 1);
    }

    const sorted = Array.from(counts.entries())
      .sort((a, b) => b[1] - a[1] || b[0].localeCompare(a[0], "ja"))
      .slice(0, limit);
    const max = sorted.reduce((current, [, count]) => Math.max(current, count), 0);

    return sorted.map(([label, count]) => ({
      label,
      value: count,
      valueText: `${count}本`,
      percent: createPercent(count, max),
    }));
  };

  const getReleaseDecadeLabel = (element: CollectionElement) => {
    const year = Number(element.sellday?.slice(0, 4));
    if (!Number.isFinite(year) || year <= 0) {
      return "発売年不明";
    }
    return `${Math.floor(year / 10) * 10}年代`;
  };

  const sumDailyPlayTimes = (
    rows: CollectionElementDailyPlayTime[],
    predicate: (date: Date) => boolean,
  ) =>
    rows.reduce((total, row) => {
      const date = parseLocalDateKey(row.playDate);
      return date && predicate(date) ? total + row.playTimeSeconds : total;
    }, 0);

  const loadDailyStats = async (elements: CollectionElement[]) => {
    const requestId = ++dailyStatsRequestId;

    if (elements.length === 0) {
      dailyPlayTimes = [];
      totalPlayTimes = [];
      isDailyStatsLoading = false;
      hasDailyStatsError = false;
      return;
    }

    isDailyStatsLoading = true;
    hasDailyStatsError = false;
    try {
      const results = await Promise.all(
        elements.map((element) =>
          commandGetCollectionElementDailyPlayTimes(element.id),
        ),
      );
      if (requestId !== dailyStatsRequestId) return;

      dailyPlayTimes = results.flat();
      totalPlayTimes = mergeDailyPlayTimes(dailyPlayTimes);
    } catch (e) {
      if (requestId !== dailyStatsRequestId) return;
      console.error("Failed to load library stats:", e);
      dailyPlayTimes = [];
      totalPlayTimes = [];
      hasDailyStatsError = true;
    } finally {
      if (requestId === dailyStatsRequestId) {
        isDailyStatsLoading = false;
      }
    }
  };

  const unsubscribeCollection = sidebarCollectionElements.subscribe((elements) => {
    void loadDailyStats(elements);
  });

  onMount(() => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });

    void sidebarCollectionElements.refetch();
  });

  onDestroy(() => {
    unsubscribeCollection();
  });

  $: elements = $sidebarCollectionElements;
  $: isInitialLoading = elements.length === 0 && $loading;
  $: totalGames = elements.length;
  $: totalPlaySeconds = elements.reduce(
    (total, element) => total + Math.max(0, element.totalPlayTimeSeconds ?? 0),
    0,
  );
  $: playedGames = elements.filter((element) => element.totalPlayTimeSeconds > 0);
  $: favoriteCount = elements.filter((element) => element.likeAt).length;
  $: brandCount = new Set(
    elements.map((element) => element.brandname).filter(Boolean),
  ).size;
  $: clearCount = elements.filter(
    (element) => element.playStatus === PlayStatus.Cleared,
  ).length;
  $: clearRate = totalGames > 0 ? Math.round((clearCount / totalGames) * 100) : 0;
  $: thisMonthSeconds = sumDailyPlayTimes(
    totalPlayTimes,
    (date) =>
      date.getFullYear() === now.getFullYear() &&
      date.getMonth() === now.getMonth(),
  );
  $: thisYearSeconds = sumDailyPlayTimes(
    totalPlayTimes,
    (date) => date.getFullYear() === now.getFullYear(),
  );
  $: heatmapSummary = buildPlayHeatmap(totalPlayTimes, now);
  $: streakStats = calculateStreakStats(totalPlayTimes);
  $: topPlayedGames = [...elements]
    .filter((element) => element.totalPlayTimeSeconds > 0)
    .sort((a, b) => b.totalPlayTimeSeconds - a.totalPlayTimeSeconds)
    .slice(0, 6);
  $: recentlyRegistered = [...elements]
    .sort((a, b) => sortByNullableDateDesc(a.registeredAt, b.registeredAt))
    .slice(0, 5);
  $: latestPlayed = [...elements]
    .filter((element) => element.lastPlayAt)
    .sort((a, b) => sortByNullableDateDesc(a.lastPlayAt, b.lastPlayAt))[0];
  $: statusStats = statusOrder.map((status): StatusStat => {
    const count = elements.filter((element) =>
      status === PlayStatus.Interrupted
        ? element.playStatus === PlayStatus.Interrupted ||
          element.playStatus === PlayStatus.LegacyShelved
        : element.playStatus === status,
    ).length;
    return {
      status,
      label: playStatusLabel[status],
      icon: playStatusIcon[status],
      count,
      percent: totalGames > 0 ? Math.round((count / totalGames) * 100) : 0,
      ...statusStyles[status],
    };
  });
  $: weekdayStats = buildWeekdayStats(totalPlayTimes);
  $: activeWeekday = [...weekdayStats].sort((a, b) => b.value - a.value)[0];
  $: registeredYearStats = buildBucketStats(
    elements.map((element) => {
      const date = parseDateTime(element.registeredAt);
      return date ? `${date.getFullYear()}年` : "登録年不明";
    }),
  );
  $: releaseDecadeStats = buildBucketStats(elements.map(getReleaseDecadeLabel));
  $: overviewCards = [
    {
      label: "登録",
      value: `${totalGames}本`,
      caption: `${playedGames.length}本プレイ済み`,
      icon: "i-material-symbols-video-library-outline-rounded",
      toneClass: "text-accent-primary bg-accent-primary/12",
    },
    {
      label: "総プレイ",
      value: formatPlayTime(totalPlaySeconds),
      caption: `今年 ${formatPlayTime(thisYearSeconds)}`,
      icon: "i-material-symbols-hourglass-outline-rounded",
      toneClass: "text-accent-warning bg-accent-warning/12",
    },
    {
      label: "今月",
      value: formatPlayTime(thisMonthSeconds),
      caption: `過去1年 ${heatmapSummary.activeDays}日`,
      icon: "i-material-symbols-calendar-today-rounded",
      toneClass: "text-accent-success bg-accent-success/12",
    },
    {
      label: "最長連続",
      value: `${streakStats.longest}日`,
      caption: `直近 ${streakStats.recent}日 / ${formatDateLabel(streakStats.latestDate)}`,
      icon: "i-material-symbols-whatshot-rounded",
      toneClass: "text-accent-error bg-accent-error/12",
    },
    {
      label: "クリア率",
      value: `${clearRate}%`,
      caption: `${clearCount}/${totalGames}本`,
      icon: "i-material-symbols-check-circle-outline-rounded",
      toneClass: "text-accent-success bg-accent-success/12",
    },
    {
      label: "お気に入り",
      value: `${favoriteCount}本`,
      caption: `${brandCount}ブランド`,
      icon: "i-material-symbols-favorite-rounded",
      toneClass: "text-accent-error bg-accent-error/12",
    },
  ] satisfies OverviewCard[];
</script>

<div class="h-full overflow-y-auto text-text-primary">
  <div class="mx-auto max-w-[1440px] space-y-6 p-4 md:p-6">
    <header class="flex flex-wrap items-end justify-between gap-4">
      <div class="min-w-0">
        <div class="mb-1 flex items-center gap-2 text-caption font-medium text-text-tertiary">
          <div class="i-material-symbols-bar-chart-rounded h-4 w-4 text-accent-primary" />
          <span>ライブラリ統計</span>
        </div>
        <h1 class="text-h1 font-bold text-text-primary">統計</h1>
      </div>
      {#if latestPlayed}
        <button
          type="button"
          class="flex max-w-full items-center gap-3 rounded-lg border border-border-primary bg-bg-secondary px-3 py-2 text-left transition-colors hover:bg-bg-tertiary"
          on:click={() =>
            push(`/works/${latestPlayed.id}?gamename=${latestPlayed.gamename}`)}
        >
          <div class="i-material-symbols-history-rounded h-5 w-5 shrink-0 text-accent-primary" />
          <div class="min-w-0">
            <div class="truncate text-sm font-semibold text-text-primary">
              {latestPlayed.gamename}
            </div>
            <div class="text-[11px] text-text-tertiary">
              最終プレイ {formatLastPlayed(latestPlayed.lastPlayAt)}
            </div>
          </div>
        </button>
      {/if}
    </header>

    {#if isInitialLoading}
      <div class="grid grid-cols-[repeat(auto-fit,minmax(180px,1fr))] gap-3">
        {#each Array(6) as _}
          <div class="rounded-lg border border-border-primary bg-bg-secondary p-4">
            <Skeleton width="36px" height="36px" className="mb-3" />
            <Skeleton width="42%" height="1.5rem" />
            <Skeleton width="70%" height="0.85rem" className="mt-2" />
          </div>
        {/each}
      </div>
    {:else if totalGames === 0}
      <div class="rounded-lg border border-border-primary bg-bg-secondary p-8 text-center">
        <div class="mx-auto mb-3 h-10 w-10 rounded-lg bg-accent-primary/12 p-2 text-accent-primary">
          <div class="i-material-symbols-bar-chart-rounded h-6 w-6" />
        </div>
        <div class="text-h3 font-bold">統計はまだありません</div>
        <div class="mt-2 text-body2 text-text-tertiary">
          ゲームを登録してプレイすると、ここにライブラリ全体の記録が集まります。
        </div>
      </div>
    {:else}
      <section
        class="grid grid-cols-[repeat(auto-fit,minmax(180px,1fr))] gap-3"
        aria-label="主要指標"
      >
        {#each overviewCards as card (card.label)}
          <div class="rounded-lg border border-border-primary bg-bg-secondary p-4">
            <div class="mb-4 flex items-center justify-between gap-3">
              <div class="text-caption font-semibold uppercase text-text-tertiary">
                {card.label}
              </div>
              <div class="flex h-9 w-9 items-center justify-center rounded-lg {card.toneClass}">
                <div class="{card.icon} h-5 w-5" />
              </div>
            </div>
            <div class="truncate text-h2 font-bold text-text-primary" title={card.value}>
              {card.value}
            </div>
            <div class="mt-1 truncate text-caption text-text-tertiary" title={card.caption}>
              {card.caption}
            </div>
          </div>
        {/each}
      </section>

      {#if isDailyStatsLoading}
        <section class="rounded-lg border border-border-primary bg-bg-secondary p-5">
          <div class="mb-4 flex items-center gap-3">
            <Skeleton width="32px" height="32px" />
            <div class="flex-1">
              <Skeleton width="180px" height="1.4rem" />
              <Skeleton width="120px" height="0.8rem" className="mt-2" />
            </div>
          </div>
          <Skeleton width="100%" height="9rem" />
        </section>
      {:else}
        <PlayHeatmap
          isTotalHeatmap={true}
          {totalPlayTimes}
          baseColor={totalHeatmapColor}
        />
      {/if}

      {#if hasDailyStatsError}
        <div class="rounded-lg border border-accent-warning/30 bg-accent-warning/8 px-4 py-3 text-body2 text-text-secondary">
          日別プレイ記録の一部を読み込めませんでした。登録数、プレイ状態、累計時間の統計は表示しています。
        </div>
      {/if}

      <section class="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <div class="rounded-lg border border-border-primary bg-bg-secondary p-5">
          <div class="mb-5 flex items-center justify-between gap-3">
            <div>
              <h2 class="text-h3 font-bold">プレイ状態</h2>
              <div class="text-caption text-text-tertiary">
                {clearCount}本クリア / {totalGames}本中
              </div>
            </div>
            <div class="i-material-symbols-checklist h-6 w-6 text-accent-primary" />
          </div>
          <div class="space-y-4">
            {#each statusStats as stat (stat.status)}
              <div>
                <div class="mb-1.5 flex items-center gap-2">
                  <div class="flex h-7 w-7 items-center justify-center rounded-md {stat.softClass}">
                    <div class="{stat.icon} h-4 w-4" />
                  </div>
                  <div class="min-w-0 flex-1 text-sm font-semibold">
                    {stat.label}
                  </div>
                  <div class="text-sm text-text-secondary">
                    {stat.count}本
                  </div>
                  <div class="w-10 text-right text-caption text-text-tertiary">
                    {stat.percent}%
                  </div>
                </div>
                <div class="h-2 overflow-hidden rounded-full bg-bg-primary">
                  <div
                    class="h-full rounded-full {stat.barClass}"
                    style="width: {stat.percent}%;"
                  />
                </div>
              </div>
            {/each}
          </div>
        </div>

        <div class="rounded-lg border border-border-primary bg-bg-secondary p-5">
          <div class="mb-5 flex items-center justify-between gap-3">
            <div>
              <h2 class="text-h3 font-bold">曜日リズム</h2>
              <div class="text-caption text-text-tertiary">
                一番長い曜日: {activeWeekday?.value ? `${activeWeekday.label}曜` : "記録なし"}
              </div>
            </div>
            <div class="i-material-symbols-calendar-month-outline-rounded h-6 w-6 text-accent-primary" />
          </div>
          <div class="space-y-3">
            {#each weekdayStats as stat (stat.label)}
              <div class="grid grid-cols-[2rem_1fr_5rem] items-center gap-3">
                <div class="text-sm font-semibold text-text-secondary">
                  {stat.label}
                </div>
                <div class="h-2.5 overflow-hidden rounded-full bg-bg-primary">
                  <div
                    class="h-full rounded-full bg-accent-primary"
                    style="width: {stat.percent}%;"
                  />
                </div>
                <div class="text-right text-caption text-text-tertiary">
                  {stat.valueText}
                </div>
              </div>
            {/each}
          </div>
        </div>
      </section>

      <section class="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
        <div class="rounded-lg border border-border-primary bg-bg-secondary p-5">
          <div class="mb-5 flex items-center justify-between gap-3">
            <div>
              <h2 class="text-h3 font-bold">よく遊んだゲーム</h2>
              <div class="text-caption text-text-tertiary">
                累計プレイ時間順
              </div>
            </div>
            <div class="i-material-symbols-whatshot-rounded h-6 w-6 text-accent-warning" />
          </div>
          {#if topPlayedGames.length === 0}
            <div class="rounded-md bg-bg-primary p-4 text-body2 text-text-tertiary">
              プレイ時間のあるゲームはまだありません。
            </div>
          {:else}
            <div class="space-y-2">
              {#each topPlayedGames as game, index (game.id)}
                <button
                  type="button"
                  class="grid w-full grid-cols-[2rem_3rem_1fr_auto] items-center gap-3 rounded-md border border-transparent bg-bg-primary/60 p-2 text-left transition-colors hover:border-border-primary hover:bg-bg-tertiary"
                  on:click={() => push(`/works/${game.id}?gamename=${game.gamename}`)}
                >
                  <div class="text-center text-sm font-bold text-text-tertiary">
                    {index + 1}
                  </div>
                  <div class="h-12 w-12 overflow-hidden rounded-md bg-bg-tertiary">
                    <img
                      src={game.thumbnail
                        ? `${convertFileSrc(game.thumbnail)}?v=${game.updatedAt}`
                        : "/images/dummy_thumbnail.svg"}
                      alt={game.gamename}
                      class="h-full w-full object-cover"
                      loading="lazy"
                      on:error={(e) => {
                        const image = e.currentTarget;
                        if (image instanceof HTMLImageElement) {
                          image.src = "/images/dummy_thumbnail.svg";
                        }
                      }}
                    />
                  </div>
                  <div class="min-w-0">
                    <div class="truncate text-sm font-semibold text-text-primary">
                      {game.gamename}
                    </div>
                    <div class="truncate text-caption text-text-tertiary">
                      {game.brandname || "ブランド不明"}
                    </div>
                  </div>
                  <div class="text-right">
                    <div class="text-sm font-semibold text-text-primary">
                      {formatPlayTime(game.totalPlayTimeSeconds)}
                    </div>
                    <div class="text-caption text-text-tertiary">
                      {formatLastPlayed(game.lastPlayAt)}
                    </div>
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <div class="space-y-4">
          <div class="rounded-lg border border-border-primary bg-bg-secondary p-5">
            <div class="mb-5 flex items-center justify-between gap-3">
              <div>
                <h2 class="text-h3 font-bold">登録した年</h2>
                <div class="text-caption text-text-tertiary">
                  最近の追加ペース
                </div>
              </div>
              <div class="i-material-symbols-library-add-check-outline-rounded h-6 w-6 text-accent-primary" />
            </div>
            <div class="space-y-3">
              {#each registeredYearStats as stat (stat.label)}
                <div class="grid grid-cols-[5rem_1fr_3.5rem] items-center gap-3">
                  <div class="truncate text-caption text-text-tertiary">
                    {stat.label}
                  </div>
                  <div class="h-2 overflow-hidden rounded-full bg-bg-primary">
                    <div
                      class="h-full rounded-full bg-accent-success"
                      style="width: {stat.percent}%;"
                    />
                  </div>
                  <div class="text-right text-caption text-text-secondary">
                    {stat.valueText}
                  </div>
                </div>
              {/each}
            </div>
          </div>

          <div class="rounded-lg border border-border-primary bg-bg-secondary p-5">
            <div class="mb-5 flex items-center justify-between gap-3">
              <div>
                <h2 class="text-h3 font-bold">発売年代</h2>
                <div class="text-caption text-text-tertiary">
                  ライブラリの世代感
                </div>
              </div>
              <div class="i-material-symbols-auto-stories-outline-rounded h-6 w-6 text-accent-warning" />
            </div>
            <div class="space-y-3">
              {#each releaseDecadeStats as stat (stat.label)}
                <div class="grid grid-cols-[5rem_1fr_3.5rem] items-center gap-3">
                  <div class="truncate text-caption text-text-tertiary">
                    {stat.label}
                  </div>
                  <div class="h-2 overflow-hidden rounded-full bg-bg-primary">
                    <div
                      class="h-full rounded-full bg-accent-warning"
                      style="width: {stat.percent}%;"
                    />
                  </div>
                  <div class="text-right text-caption text-text-secondary">
                    {stat.valueText}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      </section>

      {#if recentlyRegistered.length > 0}
        <section class="rounded-lg border border-border-primary bg-bg-secondary p-5">
          <div class="mb-4 flex items-center justify-between gap-3">
            <div>
              <h2 class="text-h3 font-bold">最近ライブラリに入ったゲーム</h2>
              <div class="text-caption text-text-tertiary">
                登録日の新しい順
              </div>
            </div>
            <div class="i-material-symbols-storefront-outline h-6 w-6 text-accent-primary" />
          </div>
          <div class="grid gap-2 md:grid-cols-2 xl:grid-cols-5">
            {#each recentlyRegistered as game (game.id)}
              <button
                type="button"
                class="min-w-0 rounded-md border border-border-primary bg-bg-primary/60 px-3 py-2 text-left transition-colors hover:bg-bg-tertiary"
                on:click={() => push(`/works/${game.id}?gamename=${game.gamename}`)}
              >
                <div class="truncate text-sm font-semibold text-text-primary">
                  {game.gamename}
                </div>
                <div class="mt-1 text-caption text-text-tertiary">
                  {parseDateTime(game.registeredAt)?.toLocaleDateString("ja-JP") ??
                    "登録日不明"}
                </div>
              </button>
            {/each}
          </div>
        </section>
      {/if}
    {/if}
  </div>
</div>
