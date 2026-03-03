<script lang="ts">
    import type { Work } from "@/lib/types";
    import { seiya } from "@/store/seiya";
    import LinkToSidebar from "@/components/Work/LinkToSidebar.svelte";
    import Detail from "@/components/Work/Detail.svelte";
    import ScreenshotGallery from "@/components/Work/ScreenshotGallery.svelte";

    export let work: Work;

    $: seiyaUrlPromise = seiya.getUrl(work.name);
</script>

<div class="grid grid-cols-1 lg:grid-cols-5 gap-8 lg:gap-12">
    <!-- Left Column: Details & Information -->
    <div class="lg:col-span-3 space-y-8">
        <!-- Links -->
        <div
            class="flex items-center gap-6 border-b border-border-primary pb-4"
        >
            <a
                href={work.officialHomePage}
                target="_blank"
                rel="noopener noreferrer"
                class="flex items-center gap-2 text-text-link hover:text-text-primary transition-colors"
            >
                Official <div class="i-material-symbols-open-in-new text-sm" />
            </a>
            <a
                href={`https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=${work.id}`}
                target="_blank"
                rel="noopener noreferrer"
                class="flex items-center gap-2 text-text-link hover:text-text-primary transition-colors"
            >
                ErogameScape <div
                    class="i-material-symbols-open-in-new text-sm"
                />
            </a>
            {#await seiyaUrlPromise then url}
                <a
                    href={url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="flex items-center gap-2 text-text-link hover:text-text-primary transition-colors"
                >
                    誠也の部屋 <div
                        class="i-material-symbols-open-in-new text-sm"
                    />
                </a>
            {/await}
        </div>

        <!-- Information -->
        <div class="space-y-6 pt-4">
            <h3 class="text-xl font-bold text-text-primary">Information</h3>

            <div
                class="rounded-xl border border-border-primary bg-bg-secondary/30 shadow-sm overflow-hidden"
            >
                <div
                    class="grid grid-cols-[120px_1fr] border-b border-border-primary"
                >
                    <div class="p-4 text-text-tertiary font-medium text-sm">
                        ブランド
                    </div>
                    <div class="p-4 text-text-link">
                        <LinkToSidebar value={work.brandName} />
                    </div>
                </div>
                <div
                    class="grid grid-cols-[120px_1fr] border-b border-border-primary"
                >
                    <div class="p-4 text-text-tertiary font-medium text-sm">
                        発売日
                    </div>
                    <div class="p-4 text-text-secondary text-sm">
                        {work.sellday}
                    </div>
                </div>
                <div
                    class="grid grid-cols-[120px_1fr] border-b border-border-primary"
                >
                    <div class="p-4 text-text-tertiary font-medium text-sm">
                        平均プレイ時間
                    </div>
                    <div class="p-4 text-text-secondary text-sm">
                        {work.statistics.playTime}
                    </div>
                </div>
                <div
                    class="grid grid-cols-[120px_1fr] border-b border-border-primary"
                >
                    <div class="p-4 text-text-tertiary font-medium text-sm">
                        中央値
                    </div>
                    <div class="p-4 text-text-secondary text-sm">
                        {work.statistics.median}
                    </div>
                </div>
                <div class="grid grid-cols-[120px_1fr]">
                    <div class="p-4 text-text-tertiary font-medium text-sm">
                        データ数
                    </div>
                    <div class="p-4 text-text-secondary text-sm">
                        {work.statistics.count}
                    </div>
                </div>
            </div>
        </div>

        <!-- Creator Details -->
        <Detail {work} />
    </div>

    <!-- Right Column: Screenshots -->
    <div class="lg:col-span-2 min-w-0">
        <ScreenshotGallery gameId={work.id} />
    </div>
</div>
