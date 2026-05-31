<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { commandGetCollectionElementDailyPlayTimes } from "@/lib/command";
    import type {
        CollectionElement,
        CollectionElementDailyPlayTime,
    } from "@/lib/types";
    import {
        buildPlayHeatmap,
        calculateDominantColorFromImageData,
        fallbackHeatmapColor,
        heatmapColorForLevel,
        parseLocalDateKey,
        type PlayHeatmapDay,
        type RgbColor,
    } from "@/lib/playHeatmap";
    import { formatPlayTime } from "@/lib/utils";

    export let element: CollectionElement | null = null;
    export let isTotalHeatmap = false;
    export let totalPlayTimes: CollectionElementDailyPlayTime[] = [];
    export let baseColor: RgbColor = fallbackHeatmapColor;

    let dailyPlayTimes: CollectionElementDailyPlayTime[] = [];
    let isLoading = true;
    let hasLoadError = false;
    let loadedElementId: number | null = null;
    let playTimeRequestId = 0;
    let loadedThumbnailSrc = "";
    let colorRequestId = 0;

    $: if (isTotalHeatmap) {
        isLoading = false;
        hasLoadError = false;
    }

    $: thumbnailSrc =
        !isTotalHeatmap && element?.thumbnail && element.thumbnail.trim() !== ""
            ? convertFileSrc(element.thumbnail)
            : "";

    $: if (!isTotalHeatmap && element?.id && element.id !== loadedElementId) {
        loadedElementId = element.id;
        baseColor = fallbackHeatmapColor;
        loadedThumbnailSrc = "";
        void loadDailyPlayTimes(element.id);
    }

    $: if (!isTotalHeatmap && thumbnailSrc && thumbnailSrc !== loadedThumbnailSrc) {
        loadedThumbnailSrc = thumbnailSrc;
        void loadThumbnailColor(thumbnailSrc);
    }

    $: heatmap = buildPlayHeatmap(isTotalHeatmap ? totalPlayTimes : dailyPlayTimes);
    $: baseColorText = `${baseColor.r}, ${baseColor.g}, ${baseColor.b}`;
    $: latestActiveDay =
        [...heatmap.days].reverse().find((day) => day.seconds > 0) ?? null;
    $: latestActiveDayText = latestActiveDay
        ? formatHeatmapDate(latestActiveDay.date)
        : "なし";

    const loadDailyPlayTimes = async (collectionElementId: number) => {
        const requestId = ++playTimeRequestId;
        isLoading = true;
        hasLoadError = false;
        try {
            const rows =
                await commandGetCollectionElementDailyPlayTimes(collectionElementId);
            if (requestId !== playTimeRequestId) return;
            dailyPlayTimes = rows;
        } catch (e) {
            if (requestId !== playTimeRequestId) return;
            console.error("Failed to load daily play times", e);
            dailyPlayTimes = [];
            hasLoadError = true;
        } finally {
            if (requestId === playTimeRequestId) {
                isLoading = false;
            }
        }
    };

    const loadThumbnailColor = async (src: string) => {
        const requestId = ++colorRequestId;
        let objectUrl = "";
        try {
            // asset:// URL を img.src に直接渡すと canvas が CORS taint され
            // getImageData が SecurityError になるため、fetch → Blob → ObjectURL 経由にする
            const response = await fetch(src);
            const blob = await response.blob();
            objectUrl = URL.createObjectURL(blob);

            const image = await new Promise<HTMLImageElement>((resolve, reject) => {
                const img = new Image();
                img.onload = () => resolve(img);
                img.onerror = () => reject(new Error("thumbnail load failed"));
                img.src = objectUrl;
            });

            const canvas = document.createElement("canvas");
            const size = 256; // 64pxのぼかしを正常に適用するためにサイズを拡張
            canvas.width = size;
            canvas.height = size;
            const context = canvas.getContext("2d", {
                willReadFrequently: true,
            });
            if (!context) return;

            // 64pxで画像をぼかすフィルターを適用
            context.filter = "blur(64px)";
            context.drawImage(image, 0, 0, size, size);
            const imageData = context.getImageData(0, 0, size, size);
            const color = calculateDominantColorFromImageData(imageData.data);
            if (requestId === colorRequestId && color) {
                baseColor = color;
            }
        } catch (e) {
            console.error("Failed to extract thumbnail color", e);
            if (requestId === colorRequestId) {
                baseColor = fallbackHeatmapColor;
            }
        } finally {
            if (objectUrl) URL.revokeObjectURL(objectUrl);
        }
    };

    const formatHeatmapDate = (dateKey: string) => {
        const date = parseLocalDateKey(dateKey);
        return date
            ? date.toLocaleDateString("ja-JP", {
                  month: "long",
                  day: "numeric",
                  weekday: "short",
              })
            : dateKey;
    };

    const getHeatmapDayLabel = (day: PlayHeatmapDay) => {
        if (day.isFuture) {
            return `${formatHeatmapDate(day.date)}: 未到来`;
        }
        if (day.seconds <= 0) {
            return `${formatHeatmapDate(day.date)}: 記録なし`;
        }
        return `${formatHeatmapDate(day.date)}: ${formatPlayTime(day.seconds)}`;
    };

    const legendLevels = [0, 1, 2, 3, 4, 5];
</script>

<section
    class="mt-8 border-t border-black/10 dark:border-white/10 pt-8"
    aria-labelledby="work-play-heatmap-title"
    style="--heatmap-rgb: {baseColorText};"
>
    <!-- ヘッダー行 -->
    <div class="flex flex-wrap items-end justify-between gap-4 mb-6">
        <div class="flex items-center gap-2.5">
            <div class="heatmap-title-icon">
                {#if isTotalHeatmap}
                    <div class="i-material-symbols-grid-view-rounded h-4 w-4" />
                {:else}
                    <div class="i-material-symbols-calendar-month-outline-rounded h-4 w-4" />
                {/if}
            </div>
            <div>
                <h3
                    id="work-play-heatmap-title"
                    class="text-h3 text-text-primary font-bold drop-shadow-sm"
                >
                    {isTotalHeatmap ? "全体のアクティビティ" : "プレイヒートマップ"}
                </h3>
                <div class="text-caption text-text-tertiary mt-0.5">
                    {isTotalHeatmap ? "すべてのゲームの合計記録" : "過去1年間の記録"}
                </div>
            </div>
        </div>

        <!-- 統計カード -->
        <div class="flex flex-wrap items-center gap-2">
            <div class="stat-card">
                <div class="stat-card-icon i-material-symbols-calendar-today-rounded" />
                <div>
                    <div class="stat-card-label">記録日</div>
                    <div class="stat-card-value">{heatmap.activeDays}日</div>
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-card-icon i-material-symbols-history-rounded" />
                <div>
                    <div class="stat-card-label">最新記録</div>
                    <div class="stat-card-value">{latestActiveDayText}</div>
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-card-icon i-material-symbols-whatshot-rounded" />
                <div>
                    <div class="stat-card-label">最長連続</div>
                    <div class="stat-card-value">{heatmap.longestStreakDays}日</div>
                </div>
            </div>
        </div>
    </div>

    <!-- ヒートマップ本体 -->
    <div class="heatmap-container">
        <!-- グリッドエリア（min-height でローディング前後の高さ変化を抑止） -->
        <div class="overflow-x-auto custom-scrollbar pb-2 heatmap-scroll-area">
            <div class="heatmap-shell">
                <div
                    class="heatmap-months"
                    style="grid-template-columns: repeat({heatmap.weekCount}, var(--heatmap-cell));"
                >
                    {#each heatmap.months as month (month.weekIndex)}
                        <div
                            class="month-label"
                            style="grid-column: {month.weekIndex + 1} / span 4;"
                        >
                            {month.label}
                        </div>
                    {/each}
                </div>
                <div class="heatmap-body">
                    <div class="weekday-labels" aria-hidden="true">
                        <span>日</span>
                        <span>月</span>
                        <span>火</span>
                        <span>水</span>
                        <span>木</span>
                        <span>金</span>
                        <span>土</span>
                    </div>
                    <div
                        class="heatmap-grid"
                        style="grid-template-columns: repeat({heatmap.weekCount}, var(--heatmap-cell));"
                        role="grid"
                        aria-label="日別プレイ記録"
                    >
                        {#each heatmap.days as day (day.date)}
                            <div
                                class="heatmap-cell"
                                class:today={day.isToday}
                                class:future={day.isFuture}
                                class:active={day.seconds > 0 && !day.isFuture}
                                role="gridcell"
                                title={getHeatmapDayLabel(day)}
                                aria-label={getHeatmapDayLabel(day)}
                                style="grid-column: {day.weekIndex + 1}; grid-row: {day.weekday + 1}; background-color: {heatmapColorForLevel(baseColor, day.level)};"
                            />
                        {/each}
                    </div>
                </div>
            </div>
        </div>

        <!-- フッター行 -->
        <div class="mt-4 flex flex-wrap items-center justify-between gap-3">
            <div class="flex items-center gap-1.5 text-caption text-text-tertiary">
                {#if isLoading}
                    <div class="i-material-symbols-progress-activity w-3.5 h-3.5 animate-spin opacity-60" />
                    <span>読み込み中…</span>
                {:else if hasLoadError}
                    <div class="i-material-symbols-error-outline-rounded w-3.5 h-3.5 opacity-60" />
                    <span>読み込み失敗</span>
                {:else}
                    <div class="i-material-symbols-check-circle-outline-rounded w-3.5 h-3.5 opacity-60" />
                    <span>{heatmap.activeDays > 0 ? `${heatmap.activeDays}日の記録` : "記録なし"}</span>
                {/if}
            </div>

            <!-- 凡例 -->
            <div class="legend-row">
                <span class="legend-text">少</span>
                <div class="legend-cells" aria-hidden="true">
                    {#each legendLevels as level}
                        <span
                            class="legend-cell"
                            class:legend-zero={level === 0}
                            style="background-color: {heatmapColorForLevel(baseColor, level)};"
                        />
                    {/each}
                </div>
                <span class="legend-text">多</span>
            </div>
        </div>
    </div>
</section>

<style>
    /* タイトルアイコン */
    .heatmap-title-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 2rem;
        height: 2rem;
        border-radius: 8px;
        background: rgba(var(--heatmap-rgb), 0.15);
        border: 1px solid rgba(var(--heatmap-rgb), 0.25);
        color: rgba(var(--heatmap-rgb), 1);
        flex-shrink: 0;
        transition: background 0.3s ease;
    }

    /* 統計カード */
    .stat-card {
        display: flex;
        align-items: center;
        gap: 0.625rem;
        padding: 0.5rem 0.875rem;
        border-radius: 10px;
        border: 1px solid rgba(var(--heatmap-rgb), 0.2);
        background: linear-gradient(
            135deg,
            rgba(var(--heatmap-rgb), 0.08) 0%,
            rgba(var(--heatmap-rgb), 0.04) 100%
        );
        backdrop-filter: blur(8px);
        transition: all 0.2s ease;
    }

    .stat-card:hover {
        border-color: rgba(var(--heatmap-rgb), 0.35);
        background: linear-gradient(
            135deg,
            rgba(var(--heatmap-rgb), 0.14) 0%,
            rgba(var(--heatmap-rgb), 0.07) 100%
        );
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(var(--heatmap-rgb), 0.12);
    }

    .stat-card-icon {
        width: 1.25rem;
        height: 1.25rem;
        flex-shrink: 0;
        color: rgb(var(--heatmap-rgb));
        opacity: 0.8;
    }

    .stat-card-label {
        font-size: 0.65rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: rgb(var(--color-text-tertiary));
        font-weight: 500;
        line-height: 1;
    }

    .stat-card-value {
        margin-top: 0.2rem;
        font-size: 0.85rem;
        font-weight: 700;
        color: rgb(var(--color-text-primary));
        line-height: 1.2;
    }

    /* ヒートマップコンテナ */
    .heatmap-container {
        border-radius: 14px;
        border: 1px solid rgba(var(--heatmap-rgb), 0.18);
        background: linear-gradient(
            145deg,
            rgba(var(--heatmap-rgb), 0.06) 0%,
            rgba(0, 0, 0, 0.04) 60%,
            rgba(var(--heatmap-rgb), 0.03) 100%
        );
        padding: 1.25rem 1.25rem 1rem;
        backdrop-filter: blur(12px);
        box-shadow:
            0 1px 0 rgba(255, 255, 255, 0.06) inset,
            0 4px 20px rgba(0, 0, 0, 0.1);
        transition: border-color 0.3s ease, box-shadow 0.3s ease;
    }

    .heatmap-container:hover {
        border-color: rgba(var(--heatmap-rgb), 0.28);
        box-shadow:
            0 1px 0 rgba(255, 255, 255, 0.07) inset,
            0 6px 24px rgba(0, 0, 0, 0.14),
            0 0 0 1px rgba(var(--heatmap-rgb), 0.06);
    }

    /* ヒートマップシェル */
    .heatmap-shell {
        --heatmap-cell: 14px;
        --heatmap-gap: 4px;
        min-width: calc((var(--heatmap-cell) + var(--heatmap-gap)) * 53 + 2rem);
    }

    .heatmap-months {
        display: grid;
        column-gap: var(--heatmap-gap);
        margin-left: 2rem;
        min-height: 1.2rem;
        margin-bottom: 0.25rem;
    }

    .month-label {
        min-width: 0;
        overflow: hidden;
        white-space: nowrap;
        font-size: 0.67rem;
        font-weight: 600;
        line-height: 1rem;
        letter-spacing: 0.03em;
        color: rgb(var(--color-text-tertiary));
        opacity: 0.8;
    }

    .heatmap-body {
        display: grid;
        grid-template-columns: 1.75rem 1fr;
        gap: 0.4rem;
        align-items: start;
    }

    .weekday-labels {
        display: grid;
        grid-template-rows: repeat(7, var(--heatmap-cell));
        gap: var(--heatmap-gap);
        color: rgb(var(--color-text-tertiary));
        font-size: 0.63rem;
        font-weight: 500;
        line-height: var(--heatmap-cell);
        text-align: right;
        opacity: 0.7;
        user-select: none;
    }

    .heatmap-grid {
        display: grid;
        grid-template-rows: repeat(7, var(--heatmap-cell));
        gap: var(--heatmap-gap);
    }

    /* セル共通 */
    .heatmap-cell,
    .legend-cell {
        border-radius: 3px;
    }

    .heatmap-cell {
        width: var(--heatmap-cell);
        height: var(--heatmap-cell);
        cursor: default;
        transition:
            transform 0.12s ease,
            box-shadow 0.12s ease,
            filter 0.12s ease;
        position: relative;
    }

    /* アクティブセル（プレイ記録あり）のホバー */
    .heatmap-cell.active:hover {
        transform: scale(1.35);
        box-shadow:
            0 0 8px rgba(var(--heatmap-rgb), 0.5),
            0 2px 6px rgba(0, 0, 0, 0.3);
        z-index: 10;
        filter: brightness(1.15);
    }

    /* 非アクティブセルのホバー */
    .heatmap-cell:not(.active):not(.future):hover {
        transform: scale(1.2);
        box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
        z-index: 10;
    }

    /* 今日のセル */
    .heatmap-cell.today {
        outline: 2px solid rgba(var(--heatmap-rgb), 0.9);
        outline-offset: 1px;
        border-radius: 3px;
    }

    /* 未来のセル */
    .heatmap-cell.future {
        opacity: 0.2;
        cursor: default;
    }

    /* 凡例 */
    .legend-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .legend-text {
        font-size: 0.67rem;
        color: rgb(var(--color-text-tertiary));
        opacity: 0.7;
        font-weight: 500;
        user-select: none;
    }

    .legend-cells {
        display: flex;
        align-items: center;
        gap: 3px;
    }

    .legend-cell {
        display: inline-block;
        width: 11px;
        height: 11px;
        transition: transform 0.15s ease;
    }

    .legend-cell:hover {
        transform: scale(1.3);
    }
</style>
