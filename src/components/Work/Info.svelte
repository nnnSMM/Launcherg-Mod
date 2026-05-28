<script lang="ts">
    import { link } from "svelte-spa-router";
    import { PlayStatus, type CollectionElement, type Work } from "@/lib/types";
    import { playStatusLabel } from "@/lib/playStatus";
    import { seiya } from "@/store/seiya";
    import LinkToSidebar from "@/components/Work/LinkToSidebar.svelte";
    import Detail from "@/components/Work/Detail.svelte";
    import ScreenshotGallery from "@/components/Work/ScreenshotGallery.svelte";
    import { formatLastPlayed, formatPlayTime, handleMarkdownClick } from "@/lib/utils";
    import { memo } from "@/store/memo";
    import { parseMarkdown } from "@/lib/markdown";


    export let work: Work;
    export let element: CollectionElement;
    export let page: "overview" | "record" | "memo" | "screenshots" =
        "overview";

    $: if (work && work.id) {
        if (!$memo.find((v) => v.workId === work.id)) {
            const localVal = localStorage.getItem(`smde_memo-${work.id}`) || "";
            memo.update((memos) => [
                ...memos.filter((m) => m.workId !== work.id),
                { workId: work.id, value: localVal, lastModified: "local" }
            ]);
        }
    }

    $: currentMemoValue = $memo.find((v) => v.workId === work.id)?.value ?? "";

    $: renderedMemoHtml = parseMarkdown(currentMemoValue);

    $: seiyaUrlPromise = work ? seiya.getUrl(work.name) : Promise.resolve("");

    const formatDate = (value: string | null | undefined) => {
        if (!value) return "未記録";
        return new Date(value).toLocaleDateString("ja-JP");
    };

    const normalizeDescription = (value: string | null | undefined) =>
        value?.trim() ?? "";

    $: recordRows = element
        ? [
        {
            label: "総プレイ時間",
            value: formatPlayTime(element.totalPlayTimeSeconds),
            icon: "i-material-symbols-hourglass-outline-rounded",
        },
        {
            label: "初プレイ",
            value: formatDate(element.firstPlayAt),
            icon: "i-material-symbols-play-circle-outline-rounded",
        },
        {
            label: "最終プレイ",
            value: formatLastPlayed(element.lastPlayAt) || formatDate(element.lastPlayAt),
            icon: "i-material-symbols-history-rounded",
        },
        {
            label: "プレイ状況",
            value: playStatusLabel[element.playStatus] ?? playStatusLabel[PlayStatus.Unplayed],
            icon: "i-material-symbols-check-circle-outline-rounded",
        },
    ]
        : [];

    $: overviewBrand = work?.brandName || element?.brandname || "ブランド未登録";
    $: overviewDescription = normalizeDescription(work?.description);
</script>

{#if work && work.id && element}
<div>
    {#if page === "overview"}
        <div class="rounded-xl border border-black/10 dark:border-white/10 bg-bg-primary/30 backdrop-blur-md shadow-md overflow-hidden relative">
            <div class="grid grid-cols-1 lg:grid-cols-[minmax(0,1fr)_minmax(300px,0.48fr)] lg:divide-x divide-black/10 dark:divide-white/10">
                <!-- 左カラム: 説明文 + 外部リンク -->
                <div class="min-w-0 flex flex-col">
                    <section
                        id="work-overview"
                        class="flex flex-col h-full"
                        aria-labelledby="work-overview-title"
                    >
                        <!-- ヘッダー（見出しとリンク） -->
                        <div class="px-5 py-4 border-b border-black/10 dark:border-white/10 flex flex-wrap items-center justify-between gap-4 bg-black/5 dark:bg-white/5">
                            <h2 id="work-overview-title" class="text-h3 text-text-primary font-bold flex items-center gap-2 drop-shadow-sm">
                                <div class="i-material-symbols-auto-stories-outline-rounded w-5 h-5 color-ui-tertiary" />
                                ストーリー
                            </h2>
                            <!-- 外部リンク -->
                            <div class="flex flex-wrap items-center gap-2">
                                <a
                                    href={work.officialHomePage}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="inline-flex items-center gap-1.5 rounded border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5 backdrop-blur-sm px-2.5 py-1 text-caption font-medium text-text-primary transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/15 hover:shadow-lg focus-visible:ring-2 focus-visible:ring-accent-accent"
                                >
                                    <div class="i-material-symbols-open-in-new w-3.5 h-3.5 opacity-80" />
                                    Official
                                </a>
                                <a
                                    href={`https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=${work.id}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="inline-flex items-center gap-1.5 rounded border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5 backdrop-blur-sm px-2.5 py-1 text-caption font-medium text-text-primary transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/15 hover:shadow-lg focus-visible:ring-2 focus-visible:ring-accent-accent"
                                >
                                    <div class="i-material-symbols-open-in-new w-3.5 h-3.5 opacity-80" />
                                    ErogameScape
                                </a>
                                {#await seiyaUrlPromise then url}
                                    <a
                                        href={url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-center gap-1.5 rounded border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5 backdrop-blur-sm px-2.5 py-1 text-caption font-medium text-text-primary transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/15 hover:shadow-lg focus-visible:ring-2 focus-visible:ring-accent-accent"
                                    >
                                        <div class="i-material-symbols-open-in-new w-3.5 h-3.5 opacity-80" />
                                        誠也の部屋
                                    </a>
                                {/await}
                            </div>
                        </div>

                        <!-- 説明文 -->
                        <div class="p-5 lg:p-7 flex-1">
                            {#if overviewDescription}
                                <p class="text-[15px] leading-8 tracking-wide text-text-primary/90 whitespace-pre-wrap break-words drop-shadow-sm">
                                    {overviewDescription}
                                </p>
                            {:else}
                                <p class="text-[13px] tracking-wide text-text-tertiary">
                                    説明文は未取得です。
                                </p>
                            {/if}
                        </div>
                    </section>
                </div>

                <!-- 右カラム: スタット + 詳細情報 -->
                <div class="min-w-0 flex flex-col divide-y divide-black/10 dark:divide-white/10">
                    <!-- スタット情報 -->
                    <section aria-label="作品情報">
                        <div class="divide-y divide-black/10 dark:divide-white/10">
                            <div class="flex items-center justify-between px-5 py-3.5 transition-colors hover:bg-black/5 dark:hover:bg-white/5 group">
                                <div class="flex items-center gap-2">
                                    <div class="i-material-symbols-storefront-outline w-4 h-4 text-text-tertiary group-hover:text-text-secondary transition-colors" />
                                    <div class="text-[11px] tracking-widest text-text-tertiary uppercase">ブランド</div>
                                </div>
                                <div class="text-body2 text-text-primary font-bold truncate min-w-0 drop-shadow-sm">
                                    <LinkToSidebar value={work.brandName} />
                                </div>
                            </div>
                            <div class="flex items-center justify-between px-5 py-3.5 transition-colors hover:bg-black/5 dark:hover:bg-white/5 group">
                                <div class="flex items-center gap-2">
                                    <div class="i-material-symbols-calendar-month-outline-rounded w-4 h-4 text-text-tertiary group-hover:text-text-secondary transition-colors" />
                                    <div class="text-[11px] tracking-widest text-text-tertiary uppercase">発売日</div>
                                </div>
                                <div class="text-body2 text-text-primary font-bold truncate min-w-0 drop-shadow-sm">
                                    {work.sellday || element.sellday || "未登録"}
                                </div>
                            </div>
                            <div class="flex items-center justify-between px-5 py-3.5 transition-colors hover:bg-black/5 dark:hover:bg-white/5 group">
                                <div class="flex items-center gap-2">
                                    <div class="i-material-symbols-hourglass-outline-rounded w-4 h-4 text-text-tertiary group-hover:text-text-secondary transition-colors" />
                                    <div class="text-[11px] tracking-widest text-text-tertiary uppercase">平均時間</div>
                                </div>
                                <div class="text-body2 text-text-primary font-bold truncate min-w-0 drop-shadow-sm">
                                    {work.statistics.playTime}
                                </div>
                            </div>
                            <div class="flex items-center justify-between px-5 py-3.5 transition-colors hover:bg-black/5 dark:hover:bg-white/5 group">
                                <div class="flex items-center gap-2">
                                    <div class="i-material-symbols-bar-chart-rounded w-4 h-4 text-text-tertiary group-hover:text-text-secondary transition-colors" />
                                    <div class="text-[11px] tracking-widest text-text-tertiary uppercase">中央値</div>
                                </div>
                                <div class="text-body2 text-text-primary font-bold truncate min-w-0 drop-shadow-sm">
                                    {work.statistics.median}
                                </div>
                            </div>
                            <div class="flex items-center justify-between px-5 py-3.5 transition-colors hover:bg-black/5 dark:hover:bg-white/5 group">
                                <div class="flex items-center gap-2">
                                    <div class="i-material-symbols-bar-chart-rounded w-4 h-4 text-text-tertiary group-hover:text-text-secondary transition-colors" />
                                    <div class="text-[11px] tracking-widest text-text-tertiary uppercase">データ数</div>
                                </div>
                                <div class="text-body2 text-text-primary font-bold truncate min-w-0 drop-shadow-sm">
                                    {work.statistics.count}件
                                </div>
                            </div>
                        </div>
                    </section>

                    <!-- 詳細情報（クリエイター等） -->
                    <section
                        class="p-5 lg:p-7 min-w-0 bg-black/5 dark:bg-white/5"
                        aria-labelledby="work-creators-title"
                    >
                        <h2 id="work-creators-title" class="text-h3 text-text-primary font-bold mb-5 flex items-center gap-2 drop-shadow-sm">
                            <div class="i-material-symbols-info-outline-rounded w-5 h-5 color-ui-tertiary" />
                            詳細情報
                        </h2>
                        <Detail {work} />
                    </section>
                </div>
            </div>
        </div>
    {:else if page === "record"}
        <section
            id="work-record"
            class="rounded-xl border border-black/10 dark:border-white/10 bg-bg-primary/30 backdrop-blur-md shadow-md p-5 lg:p-7"
            aria-labelledby="work-record-title"
        >
            <div class="flex items-center gap-3 mb-6">
                <div class="i-material-symbols-history-rounded w-6 h-6 color-ui-tertiary drop-shadow-sm" />
                <h2 id="work-record-title" class="text-h3 text-text-primary font-bold drop-shadow-sm">
                    記録
                </h2>
            </div>
            <div class="mb-8 flex flex-wrap items-center gap-x-6 gap-y-4">
                {#each recordRows as row (row.label)}
                    <div class="flex min-w-[10rem] items-center gap-3 rounded-lg border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5 backdrop-blur-sm px-3 py-2 transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/10 hover:shadow-md">
                        <div class="{row.icon} h-7 w-7 shrink-0 color-ui-tertiary drop-shadow-sm" />
                        <div class="min-w-0">
                            <div class="text-[11px] tracking-widest text-text-tertiary uppercase">
                                {row.label}
                            </div>
                            <div class="mt-0.5 max-w-44 truncate text-body2 font-bold text-text-primary drop-shadow-sm">
                                {row.value}
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
            <div class="rounded-lg border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5 backdrop-blur-sm p-5">
                <div class="text-[11px] tracking-widest text-text-tertiary uppercase mb-4 flex items-center gap-1.5">
                    <div class="i-material-symbols-public-rounded w-4 h-4 opacity-80" />
                    コミュニティ統計
                </div>
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <div class="min-w-0 flex flex-col gap-1.5">
                        <div class="text-[10px] tracking-widest text-text-tertiary uppercase">平均プレイ時間</div>
                        <div class="rounded border border-black/5 dark:border-white/5 bg-black/5 dark:bg-white/5 px-3 py-2 text-body2 text-text-primary font-bold truncate backdrop-blur-sm drop-shadow-sm">
                            {work.statistics.playTime}
                        </div>
                    </div>
                    <div class="min-w-0 flex flex-col gap-1.5">
                        <div class="text-[10px] tracking-widest text-text-tertiary uppercase">中央値</div>
                        <div class="rounded border border-black/5 dark:border-white/5 bg-black/5 dark:bg-white/5 px-3 py-2 text-body2 text-text-primary font-bold truncate backdrop-blur-sm drop-shadow-sm">
                            {work.statistics.median}
                        </div>
                    </div>
                    <div class="min-w-0 flex flex-col gap-1.5">
                        <div class="text-[10px] tracking-widest text-text-tertiary uppercase">データ数</div>
                        <div class="rounded border border-black/5 dark:border-white/5 bg-black/5 dark:bg-white/5 px-3 py-2 text-body2 text-text-primary font-bold truncate backdrop-blur-sm drop-shadow-sm">
                            {work.statistics.count}
                        </div>
                    </div>
                </div>
            </div>
        </section>
    {:else if page === "memo"}
        <section
            id="work-memo"
            class="rounded-xl border border-black/10 dark:border-white/10 bg-bg-primary/30 backdrop-blur-md shadow-md p-5 lg:p-7 transition-all duration-300"
            aria-labelledby="work-memo-title"
        >
            <div class="flex items-start gap-4">
                <div class="i-material-symbols-drive-file-rename-outline w-6 h-6 color-ui-tertiary shrink-0 mt-1 drop-shadow-sm" />
                <div class="min-w-0 flex-1">
                    <h2 id="work-memo-title" class="text-h3 text-text-primary font-bold drop-shadow-sm">
                        Memo
                    </h2>
                    <a
                        href={`/memos/${work.id}?gamename=${encodeURIComponent(element.gamename)}`}
                        use:link
                        class="mt-4 inline-flex items-center gap-2 rounded border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5 backdrop-blur-md px-3 py-1.5 text-[13px] font-medium text-text-primary transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/15 hover:shadow-lg focus-visible:ring-2 focus-visible:ring-accent-accent drop-shadow-sm"
                    >
                        <div class="i-material-symbols-open-in-new-rounded w-4 h-4 opacity-80" />
                        メモを開く
                    </a>

                    <div class="mt-6 border-t border-black/10 dark:border-white/10 pt-6">
                        <h3 class="text-[11px] tracking-widest text-text-tertiary uppercase mb-4 flex items-center gap-2 font-bold">
                            <div class="i-material-symbols-auto-stories-outline-rounded w-4 h-4 color-ui-tertiary" />
                            内容
                        </h3>
                        <div class="rounded-xl border border-black/10 dark:border-white/10 bg-bg-secondary/40 backdrop-blur-lg p-6 text-text-primary overflow-auto max-h-[600px] shadow-lg inset-shadow-sm transition-all duration-300 hover:border-black/15 dark:hover:border-white/15">
                            {#if renderedMemoHtml}
                                <div class="markdown-body" on:click={handleMarkdownClick}>
                                    {@html renderedMemoHtml}
                                </div>
                            {:else}
                                <div class="text-caption text-text-tertiary italic flex items-center gap-2">
                                    <div class="i-material-symbols-edit-note-rounded w-4 h-4 opacity-60" />
                                    メモは空です。「メモを開く」ボタンから編集できます。
                                </div>
                            {/if}
                        </div>
                    </div>
                </div>
            </div>
        </section>
    {:else if page === "screenshots"}
        <section
            id="work-screenshots"
            class="rounded-xl border border-black/10 dark:border-white/10 bg-bg-primary/30 backdrop-blur-md shadow-md p-5 lg:p-7"
            aria-label="スクリーンショット"
        >
            <ScreenshotGallery gameId={work.id} />
        </section>
    {/if}
</div>
{/if}
