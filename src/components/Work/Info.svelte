<script lang="ts">
    import { link } from "svelte-spa-router";
    import { PlayStatus, type CollectionElement, type Work } from "@/lib/types";
    import { playStatusLabel } from "@/lib/playStatus";
    import { seiya } from "@/store/seiya";
    import LinkToSidebar from "@/components/Work/LinkToSidebar.svelte";
    import Detail from "@/components/Work/Detail.svelte";
    import ScreenshotGallery from "@/components/Work/ScreenshotGallery.svelte";
    import { formatLastPlayed, formatPlayTime } from "@/lib/utils";

    export let work: Work;
    export let element: CollectionElement;
    export let page: "overview" | "record" | "memo" | "screenshots" =
        "overview";

    $: seiyaUrlPromise = seiya.getUrl(work.name);

    const formatDate = (value: string | null | undefined) => {
        if (!value) return "未記録";
        return new Date(value).toLocaleDateString("ja-JP");
    };

    const normalizeDescription = (value: string | null | undefined) =>
        value
            ?.replace(/[ \t]+/g, " ")
            .replace(/\n{3,}/g, "\n\n")
            .trim() ?? "";

    $: recordRows = [
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
    ];

    $: overviewBrand = work.brandName || element.brandname || "ブランド未登録";
    $: overviewDescription = normalizeDescription(work.description);
</script>

<div>
    {#if page === "overview"}
        <div class="grid grid-cols-1 2xl:grid-cols-[minmax(0,1fr)_minmax(340px,0.52fr)] gap-5 lg:gap-6">
            <div class="min-w-0">
                <section
                    id="work-overview"
                    class="rounded-lg border border-border-primary bg-bg-primary/38 backdrop-blur-md shadow-sm overflow-hidden"
                    aria-labelledby="work-overview-title"
                >
                    <div class="p-4 lg:p-5 border-b border-border-primary">
                        <div class="flex flex-col gap-3 lg:flex-row lg:items-start">
                            <div class="min-w-0 flex-1">
                                <h2 id="work-overview-title" class="text-h3 text-text-primary font-bold">
                                    概要
                                </h2>
                                <p class="text-body3 text-text-tertiary mt-1 truncate">
                                    {overviewBrand}
                                </p>
                                {#if overviewDescription}
                                    <p class="mt-4 max-w-4xl text-body2 leading-7 text-text-secondary whitespace-pre-line break-words">
                                        {overviewDescription}
                                    </p>
                                {:else}
                                    <p class="mt-4 text-body3 text-text-tertiary">
                                        説明文は未取得です。
                                    </p>
                                {/if}
                            </div>
                            <div class="flex flex-wrap gap-2">
                                <a
                                    href={work.officialHomePage}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="inline-flex items-center gap-1 rounded border border-border-primary px-3 py-1.5 text-body3 text-text-link hover:bg-bg-button-hover hover:text-text-primary focus-visible:ring-2 focus-visible:ring-accent-accent"
                                >
                                    Official
                                    <div class="i-material-symbols-open-in-new text-sm" />
                                </a>
                                <a
                                    href={`https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=${work.id}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="inline-flex items-center gap-1 rounded border border-border-primary px-3 py-1.5 text-body3 text-text-link hover:bg-bg-button-hover hover:text-text-primary focus-visible:ring-2 focus-visible:ring-accent-accent"
                                >
                                    ErogameScape
                                    <div class="i-material-symbols-open-in-new text-sm" />
                                </a>
                                {#await seiyaUrlPromise then url}
                                    <a
                                        href={url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-center gap-1 rounded border border-border-primary px-3 py-1.5 text-body3 text-text-link hover:bg-bg-button-hover hover:text-text-primary focus-visible:ring-2 focus-visible:ring-accent-accent"
                                    >
                                        誠也の部屋
                                        <div class="i-material-symbols-open-in-new text-sm" />
                                    </a>
                                {/await}
                            </div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3">
                        <div class="p-4 border-b md:border-r border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">ブランド</div>
                            <div class="text-body2 text-text-link font-medium truncate">
                                <LinkToSidebar value={work.brandName} />
                            </div>
                        </div>
                        <div class="p-4 border-b xl:border-r border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">発売日</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {work.sellday || element.sellday || "未登録"}
                            </div>
                        </div>
                        <div class="p-4 border-b md:border-r xl:border-r-0 border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">平均プレイ時間</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {work.statistics.playTime}
                            </div>
                        </div>
                        <div class="p-4 border-b xl:border-b-0 xl:border-r border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">中央値</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {work.statistics.median}
                            </div>
                        </div>
                        <div class="p-4 border-b md:border-b-0 md:border-r border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">データ数</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {work.statistics.count}
                            </div>
                        </div>
                        <div class="p-4 min-w-0">
                            <div class="text-caption text-text-tertiary">属性</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {element.isNukige ? "抜きゲー" : "通常"}
                            </div>
                        </div>
                    </div>
                </section>
            </div>

            <section
                class="rounded-lg border border-border-primary bg-bg-primary/24 backdrop-blur-md shadow-sm p-4 lg:p-5 min-w-0"
                aria-labelledby="work-creators-title"
            >
                <h2 id="work-creators-title" class="text-h3 text-text-primary font-bold mb-4">
                    詳細情報
                </h2>
                <Detail {work} />
            </section>
        </div>
    {:else if page === "record"}
        <section
            id="work-record"
            class="rounded-lg border border-border-primary bg-bg-primary/30 backdrop-blur-md shadow-sm p-4 lg:p-5"
            aria-labelledby="work-record-title"
        >
            <div class="flex items-center gap-2 mb-4">
                <div class="i-material-symbols-history-rounded w-5 h-5 color-ui-tertiary" />
                <h2 id="work-record-title" class="text-h3 text-text-primary font-bold">
                    記録
                </h2>
            </div>
            <div class="mb-6 flex flex-wrap items-center gap-x-10 gap-y-5">
                {#each recordRows as row (row.label)}
                    <div class="flex min-w-[10rem] items-center gap-3 rounded-lg border border-border-primary bg-bg-secondary/10 backdrop-blur-sm px-3 py-2">
                        <div class="{row.icon} h-7 w-7 shrink-0 color-ui-tertiary" />
                        <div class="min-w-0">
                            <div class="text-caption text-text-tertiary">
                                {row.label}
                            </div>
                            <div class="mt-1 max-w-44 truncate text-body2 font-semibold text-text-primary">
                                {row.value}
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
            <div class="rounded-lg border border-border-primary bg-bg-secondary/5 backdrop-blur-sm p-4">
                <div class="text-caption text-text-tertiary">コミュニティ統計</div>
                <div class="mt-3 grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <div class="min-w-0">
                        <div class="text-caption text-text-tertiary">平均プレイ時間</div>
                        <div class="mt-1 rounded-md bg-bg-secondary/5 px-2.5 py-2 text-body2 text-text-primary font-semibold truncate backdrop-blur-sm">
                            {work.statistics.playTime}
                        </div>
                    </div>
                    <div class="min-w-0">
                        <div class="text-caption text-text-tertiary">中央値</div>
                        <div class="mt-1 rounded-md bg-bg-secondary/5 px-2.5 py-2 text-body2 text-text-primary font-semibold truncate backdrop-blur-sm">
                            {work.statistics.median}
                        </div>
                    </div>
                    <div class="min-w-0">
                        <div class="text-caption text-text-tertiary">データ数</div>
                        <div class="mt-1 rounded-md bg-bg-secondary/5 px-2.5 py-2 text-body2 text-text-primary font-semibold truncate backdrop-blur-sm">
                            {work.statistics.count}
                        </div>
                    </div>
                </div>
            </div>
        </section>
    {:else if page === "memo"}
        <section
            id="work-memo"
            class="rounded-lg border border-border-primary bg-bg-primary/30 backdrop-blur-md shadow-sm p-4 lg:p-5"
            aria-labelledby="work-memo-title"
        >
            <div class="flex items-start gap-3">
                <div class="i-material-symbols-drive-file-rename-outline w-6 h-6 color-ui-tertiary shrink-0 mt-1" />
                <div class="min-w-0 flex-1">
                    <h2 id="work-memo-title" class="text-h3 text-text-primary font-bold">
                        Memo
                    </h2>
                    <p class="text-body3 text-text-tertiary mt-1">
                        既存のメモ画面を開きます。保存方式とスクリーンショット貼り付けは従来どおりです。
                    </p>
                    <a
                        href={`/memos/${work.id}?gamename=${encodeURIComponent(element.gamename)}`}
                        use:link
                        class="mt-4 inline-flex items-center gap-2 rounded bg-bg-button px-3 py-2 text-body2 text-text-primary hover:bg-bg-button-hover focus-visible:ring-2 focus-visible:ring-accent-accent"
                    >
                        <div class="i-material-symbols-open-in-new-rounded w-4 h-4" />
                        メモを開く
                    </a>
                </div>
            </div>
        </section>
    {:else if page === "screenshots"}
        <section
            id="work-screenshots"
            class="rounded-lg border border-border-primary bg-bg-primary/72 shadow-sm p-4 lg:p-5"
            aria-label="スクリーンショット"
        >
            <ScreenshotGallery gameId={work.id} />
        </section>
    {/if}
</div>
