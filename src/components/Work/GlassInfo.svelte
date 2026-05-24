<script lang="ts">
    import Actions from "@/components/Work/Actions.svelte";
    import { seiya } from "@/store/seiya";
    import Info from "@/components/Work/Info.svelte";
    import { PlayStatus, type Work, type CollectionElement } from "@/lib/types";
    import type { PlayStatus as PlayStatusType } from "@/lib/types";
    import { playStatusIcon, playStatusLabel } from "@/lib/playStatus";
    import { formatLastPlayed, formatPlayTime } from "@/lib/utils";
    import Select from "@/components/UI/Select.svelte";
    import APopover from "@/components/UI/APopover.svelte";
    import {
        commandAdjustUntrackedPlayTimeSeconds,
        commandUpdateElementPlayStatus,
    } from "@/lib/command";
    import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
    import { showErrorToast, showInfoToast } from "@/lib/toast";

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
    let isAddingUntrackedPlayTime = false;
    let playTimeAdjustmentInput = "";

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

    const formatDateTime = (value: string | null | undefined) => {
        if (!value) return "未記録";
        return new Date(value).toLocaleString("ja-JP", {
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
        });
    };

    const parsePlayTimeAdjustmentSeconds = (value: string) => {
        const trimmed = value.trim();
        if (!trimmed) return null;

        const parts = trimmed.split(":");
        if (parts.length > 2) return null;

        const hours = Number(parts[0]);
        const minutes = parts.length === 2 ? Number(parts[1]) : 0;
        if (!Number.isFinite(hours) || !Number.isFinite(minutes)) return null;
        if (hours < 0 || minutes < 0) return null;

        return Math.floor(hours) * 3600 + Math.floor(minutes) * 60;
    };

    const formatAdjustmentInput = (seconds: number) => {
        const totalMinutes = Math.floor(seconds / 60);
        const hours = Math.floor(totalMinutes / 60);
        const minutes = totalMinutes % 60;
        return `${hours}:${minutes.toString().padStart(2, "0")}`;
    };

    const normalizePlayTimeAdjustmentInput = () => {
        const seconds = parsePlayTimeAdjustmentSeconds(playTimeAdjustmentInput);
        if (seconds === null) return;
        playTimeAdjustmentInput = formatAdjustmentInput(seconds);
    };

    const adjustUntrackedPlayTime = async (
        direction: 1 | -1,
        close: (_?: unknown) => void,
    ) => {
        if (isAddingUntrackedPlayTime) return;

        const parsedSeconds = parsePlayTimeAdjustmentSeconds(playTimeAdjustmentInput);
        if (parsedSeconds === null || parsedSeconds <= 0) {
            showInfoToast("計上する時間を H:MM 形式で入力してください");
            return;
        }

        const signedSeconds = parsedSeconds * direction;
        const appliedSeconds =
            direction < 0
                ? -Math.min(parsedSeconds, element.totalPlayTimeSeconds)
                : signedSeconds;

        if (appliedSeconds === 0) {
            showInfoToast("差し引けるプレイ時間がありません");
            return;
        }

        try {
            isAddingUntrackedPlayTime = true;
            await commandAdjustUntrackedPlayTimeSeconds(element.id, signedSeconds);

            const playedAt = new Date().toISOString();
            element = {
                ...element,
                firstPlayAt:
                    appliedSeconds > 0 ? element.firstPlayAt ?? playedAt : element.firstPlayAt,
                lastPlayAt: appliedSeconds > 0 ? playedAt : element.lastPlayAt,
                totalPlayTimeSeconds: Math.max(
                    0,
                    element.totalPlayTimeSeconds + appliedSeconds,
                ),
            };
            showInfoToast(
                appliedSeconds > 0
                    ? `${formatPlayTime(appliedSeconds)}をプレイ時間に計上しました`
                    : `${formatPlayTime(-appliedSeconds)}をプレイ時間から差し引きました`,
            );
            playTimeAdjustmentInput = formatAdjustmentInput(parsedSeconds);
            close();
        } catch (e) {
            showErrorToast(e as string);
        } finally {
            isAddingUntrackedPlayTime = false;
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
    $: adjustmentSeconds = parsePlayTimeAdjustmentSeconds(playTimeAdjustmentInput);
    $: hasValidAdjustmentSeconds = adjustmentSeconds !== null && adjustmentSeconds > 0;
    $: addedTotalPlayTimeText = hasValidAdjustmentSeconds
        ? formatPlayTime(element.totalPlayTimeSeconds + (adjustmentSeconds ?? 0))
        : playTimeText;
    $: subtractedTotalPlayTimeText = hasValidAdjustmentSeconds
        ? formatPlayTime(Math.max(0, element.totalPlayTimeSeconds - (adjustmentSeconds ?? 0)))
        : playTimeText;
</script>

<div
    class="relative z-20 bg-bg-primary/28 border-t border-border-primary p-4 sm:p-6 lg:p-8 shadow-2xl"
    style="backdrop-filter: blur(8px);"
>
    <div class="max-w-[1440px] mx-auto space-y-8">
        <div class="space-y-8" aria-label="作品操作">
            <div class="min-w-0 px-1 lg:px-2">
                {#await seiya.getUrl(work.name)}
                    <Actions id={work.id} seiyaUrl={""} />
                {:then seiyaUrl}
                    <Actions id={work.id} {seiyaUrl} />
                {/await}
            </div>
            <div class="min-w-0 px-1 lg:px-2">
                <div class="min-w-0 flex flex-wrap items-center gap-x-6 gap-y-4">
                    <div class="flex min-w-0 items-center gap-3">
                        <Select
                            options={playStatusOptions}
                            bind:value={currentPlayStatus}
                            on:select={handlePlayStatusSelect}
                            showSelectedCheck={true}
                            showSelectedBackground={false}
                            popoverPlacement="top"
                            title="プレイ状況を変更"
                        >
                            <div
                                class="group h-8 w-8 shrink-0 cursor-pointer"
                                aria-label={`プレイ状況を変更: ${currentStatusLabel}`}
                                title="プレイ状況を変更"
                            >
                                <div
                                    class="{currentStatusIcon} h-8 w-8 {currentStatusIconTone} transition-colors group-hover:color-text-primary"
                                />
                            </div>
                        </Select>
                        <div class="w-24 min-w-0">
                            <div class="text-[12px] leading-none text-text-tertiary">状態</div>
                            <div class="mt-1.5 truncate text-body2 font-semibold text-text-primary">
                                {currentStatusLabel}
                            </div>
                        </div>
                    </div>
                    <div class="flex min-w-0 items-center gap-3">
                        <APopover let:close panelClass="w-80" placement="top">
                            <div
                                slot="button"
                                class="group h-8 w-8 shrink-0 cursor-pointer"
                                aria-label="プレイ時間の詳細と未記録分の計上"
                                title="プレイ時間の詳細"
                            >
                                <div
                                    class="i-material-symbols-timer-outline-rounded h-8 w-8 color-ui-tertiary transition-colors group-hover:color-text-primary"
                                />
                            </div>
                            <div class="w-80 max-w-[calc(100vw-16px)] p-4">
                                <div class="text-body2 font-semibold text-text-primary">
                                    プレイ時間
                                </div>
                                <div class="mt-3 grid gap-2 text-body3">
                                    <div class="flex items-center justify-between gap-4">
                                        <span class="text-text-tertiary">総プレイ時間</span>
                                        <span class="font-semibold text-text-primary">
                                            {playTimeText}
                                        </span>
                                    </div>
                                    <div class="flex items-center justify-between gap-4">
                                        <span class="text-text-tertiary">初プレイ</span>
                                        <span class="truncate text-right font-medium text-text-secondary">
                                            {formatDateTime(element.firstPlayAt)}
                                        </span>
                                    </div>
                                    <div class="flex items-center justify-between gap-4">
                                        <span class="text-text-tertiary">最終プレイ</span>
                                        <span class="truncate text-right font-medium text-text-secondary">
                                            {formatDateTime(element.lastPlayAt)}
                                        </span>
                                    </div>
                                </div>
                                <div class="mt-4 border-t border-border-primary pt-3">
                                    <div class="text-caption font-semibold text-text-tertiary">
                                        未記録分を調整
                                    </div>
                                    <div class="mt-2 flex items-center gap-2">
                                        <input
                                            bind:value={playTimeAdjustmentInput}
                                            type="text"
                                            inputmode="numeric"
                                            placeholder="例: 2:00"
                                            disabled={isAddingUntrackedPlayTime}
                                            class="h-8 min-w-0 flex-1 rounded border border-border-primary bg-bg-primary px-2 text-body3 font-semibold text-text-primary outline-none transition-colors focus:border-accent-accent disabled:cursor-not-allowed disabled:opacity-60"
                                            on:blur={normalizePlayTimeAdjustmentInput}
                                            on:keydown={(event) => {
                                                if (event.key === "Enter") {
                                                    normalizePlayTimeAdjustmentInput();
                                                }
                                            }}
                                        />
                                        <button
                                            type="button"
                                            disabled={isAddingUntrackedPlayTime}
                                            class="h-8 rounded bg-bg-button px-2.5 text-caption font-semibold text-text-primary transition-colors hover:bg-bg-button-hover disabled:cursor-not-allowed disabled:opacity-60"
                                            on:click={() => adjustUntrackedPlayTime(1, close)}
                                        >
                                            計上
                                        </button>
                                        <button
                                            type="button"
                                            disabled={isAddingUntrackedPlayTime}
                                            class="h-8 rounded bg-bg-button px-2.5 text-caption font-semibold text-text-primary transition-colors hover:bg-bg-button-hover disabled:cursor-not-allowed disabled:opacity-60"
                                            on:click={() => adjustUntrackedPlayTime(-1, close)}
                                        >
                                            差し引く
                                        </button>
                                    </div>
                                    <div class="mt-3 grid grid-cols-2 gap-2">
                                        <div class="flex items-center justify-between gap-3">
                                            <span class="text-caption font-medium text-text-tertiary">計上後</span>
                                            <span class="text-body3 font-semibold text-text-primary">
                                                {addedTotalPlayTimeText}
                                            </span>
                                        </div>
                                        <div class="flex items-center justify-between gap-3">
                                            <span class="text-caption font-medium text-text-tertiary">差引後</span>
                                            <span class="text-body3 font-semibold text-text-primary">
                                                {subtractedTotalPlayTimeText}
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </APopover>
                        <div class="w-24 min-w-0">
                            <div class="text-[12px] leading-none text-text-tertiary">プレイ時間</div>
                            <div class="mt-1.5 truncate text-body2 font-semibold text-text-primary">
                                {playTimeText}
                            </div>
                        </div>
                    </div>
                    <div class="flex min-w-0 items-center gap-3">
                        <div class="i-material-symbols-update-rounded h-8 w-8 shrink-0 color-ui-tertiary" />
                        <div class="w-24 min-w-0">
                            <div class="text-[12px] leading-none text-text-tertiary">最終プレイ</div>
                            <div class="mt-1.5 truncate text-body2 font-semibold text-text-primary">
                                {lastPlayedText || "未記録"}
                            </div>
                        </div>
                    </div>
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
