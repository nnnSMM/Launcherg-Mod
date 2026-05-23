<script lang="ts">
    import Actions from "@/components/Work/Actions.svelte";
    import { seiya } from "@/store/seiya";
    import Info from "@/components/Work/Info.svelte";
    import { PlayStatus, type Work, type CollectionElement } from "@/lib/types";
    import type { PlayStatus as PlayStatusType } from "@/lib/types";
    import { playStatusIcon, playStatusLabel } from "@/lib/playStatus";
    import { formatLastPlayed, formatPlayTime } from "@/lib/utils";
    import Select from "@/components/UI/Select.svelte";
    import { commandUpdateElementPlayStatus } from "@/lib/command";
    import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
    import { showErrorToast } from "@/lib/toast";

    export let work: Work;
    export let element: CollectionElement;

    type DetailPage = "overview" | "record" | "memo" | "screenshots";

    const statusIconTone = {
        [PlayStatus.Unplayed]: "color-ui-tertiary",
        [PlayStatus.Playing]: "color-accent-primary",
        [PlayStatus.Cleared]: "color-accent-success",
        [PlayStatus.Multiple]: "color-accent-warning",
        [PlayStatus.Shelved]: "color-ui-tertiary",
    };

    const pageTabs: {
        id: DetailPage;
        label: string;
    }[] = [
        {
            id: "overview",
            label: "概要",
        },
        {
            id: "record",
            label: "記録",
        },
        {
            id: "memo",
            label: "メモ",
        },
        {
            id: "screenshots",
            label: "スクリーンショット",
        },
    ];

    const playStatusOptions: { label: string; value: PlayStatusType }[] = [
        {
            label: playStatusLabel[PlayStatus.Unplayed],
            value: PlayStatus.Unplayed,
        },
        {
            label: playStatusLabel[PlayStatus.Playing],
            value: PlayStatus.Playing,
        },
        {
            label: playStatusLabel[PlayStatus.Cleared],
            value: PlayStatus.Cleared,
        },
        {
            label: playStatusLabel[PlayStatus.Multiple],
            value: PlayStatus.Multiple,
        },
        {
            label: playStatusLabel[PlayStatus.Shelved],
            value: PlayStatus.Shelved,
        },
    ];

    let activePage: DetailPage = "overview";

    const handlePlayStatusSelect = (
        event: CustomEvent<{ value: string | number }>,
    ) => {
        updatePlayStatus(event.detail.value as PlayStatusType);
    };

    const updatePlayStatus = async (newStatus: PlayStatusType) => {
        if (newStatus === element.playStatus) return;

        try {
            await commandUpdateElementPlayStatus(element.id, newStatus);
            sidebarCollectionElements.updatePlayStatus(element.id, newStatus);
            element = { ...element, playStatus: newStatus };
        } catch (e) {
            showErrorToast(e as string);
        }
    };

    $: currentPlayStatus = element.playStatus;
    $: currentStatusLabel =
        playStatusLabel[element.playStatus] ?? playStatusLabel[PlayStatus.Unplayed];
    $: currentStatusIcon =
        playStatusIcon[element.playStatus] ?? playStatusIcon[PlayStatus.Unplayed];
    $: currentStatusIconTone =
        statusIconTone[element.playStatus] ?? statusIconTone[PlayStatus.Unplayed];
    $: lastPlayedText = formatLastPlayed(element.lastPlayAt);
    $: playTimeText = formatPlayTime(element.totalPlayTimeSeconds);
</script>

<div
    class="relative z-20 bg-bg-primary/48 border-t border-border-primary p-4 sm:p-6 lg:p-8 shadow-2xl"
    style="backdrop-filter: blur(8px);"
>
    <div class="max-w-[1440px] mx-auto space-y-6">
        <div class="space-y-4" aria-label="作品操作">
            <div class="min-w-0 px-1 lg:px-2">
                {#await seiya.getUrl(work.name)}
                    <Actions id={work.id} seiyaUrl={""} />
                {:then seiyaUrl}
                    <Actions id={work.id} {seiyaUrl} />
                {/await}
            </div>
            <div class="min-w-0 px-1 lg:px-2">
                <div class="min-w-0 flex flex-wrap items-center gap-x-5 gap-y-3">
                    <div class="flex min-w-0 items-center gap-2.5">
                        <Select
                            options={playStatusOptions}
                            bind:value={currentPlayStatus}
                            on:select={handlePlayStatusSelect}
                            showSelectedCheck={true}
                            title="プレイ状況を変更"
                        >
                            <div
                                class="group h-7 w-7 shrink-0 cursor-pointer"
                                aria-label={`プレイ状況を変更: ${currentStatusLabel}`}
                                title="プレイ状況を変更"
                            >
                                <div
                                    class="{currentStatusIcon} h-7 w-7 {currentStatusIconTone} transition-colors group-hover:color-text-primary"
                                />
                            </div>
                        </Select>
                        <div class="w-20 min-w-0">
                            <div class="text-[11px] leading-none text-text-tertiary">状態</div>
                            <div class="mt-1 truncate text-body3 font-semibold text-text-primary">
                                {currentStatusLabel}
                            </div>
                        </div>
                    </div>
                    <div class="flex min-w-0 items-center gap-2.5">
                        <div class="i-material-symbols-hourglass-outline-rounded h-7 w-7 shrink-0 color-ui-tertiary" />
                        <div class="w-[4.75rem] min-w-0">
                            <div class="text-[11px] leading-none text-text-tertiary">プレイ時間</div>
                            <div class="mt-1 truncate text-body3 font-semibold text-text-primary">
                                {playTimeText}
                            </div>
                        </div>
                    </div>
                    <div class="flex min-w-0 items-center gap-2.5">
                        <div class="i-material-symbols-history-rounded h-7 w-7 shrink-0 color-ui-tertiary" />
                        <div class="w-[4.75rem] min-w-0">
                            <div class="text-[11px] leading-none text-text-tertiary">最終プレイ</div>
                            <div class="mt-1 truncate text-body3 font-semibold text-text-primary">
                                {lastPlayedText || "未記録"}
                            </div>
                        </div>
                    </div>
                    {#if element.sellday}
                        <div class="flex min-w-0 items-center gap-2.5">
                            <div class="i-material-symbols-calendar-month-outline-rounded h-7 w-7 shrink-0 color-ui-tertiary" />
                            <div class="w-[5.75rem] min-w-0">
                                <div class="text-[11px] leading-none text-text-tertiary">発売日</div>
                                <div class="mt-1 truncate text-body3 font-semibold text-text-primary">
                                    {element.sellday}
                                </div>
                            </div>
                        </div>
                    {/if}
                    {#if element.brandname}
                        <div class="flex min-w-0 items-center gap-2.5">
                            <div class="i-material-symbols-business-rounded h-7 w-7 shrink-0 color-ui-tertiary" />
                            <div class="min-w-0">
                                <div class="text-[11px] leading-none text-text-tertiary">ブランド</div>
                                <div class="mt-1 text-body3 font-semibold text-text-primary">
                                    {element.brandname}
                                </div>
                            </div>
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        <div
            class="grid grid-cols-4 border-b border-border-primary"
            aria-label="作品詳細ページ"
            role="tablist"
        >
            {#each pageTabs as tab (tab.id)}
                <button
                    type="button"
                    role="tab"
                    aria-selected={activePage === tab.id}
                    class:border-accent-accent={activePage === tab.id}
                    class:border-transparent={activePage !== tab.id}
                    class:text-text-primary={activePage === tab.id}
                    class:text-text-secondary={activePage !== tab.id}
                    class="inline-flex h-11 w-full min-w-0 cursor-pointer items-center justify-center whitespace-nowrap border-b-2 bg-transparent px-1 text-body3 font-medium transition-colors hover:text-text-primary focus-visible:ring-2 focus-visible:ring-accent-accent"
                    on:click={() => (activePage = tab.id)}
                >
                    <span>{tab.label}</span>
                </button>
            {/each}
        </div>

        <Info {work} {element} page={activePage} />
    </div>
</div>
