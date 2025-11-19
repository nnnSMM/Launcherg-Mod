<script lang="ts">
    import Detail from "@/components/Work/Detail.svelte";
    import Actions from "@/components/Work/Actions.svelte";
    import type { Work, CollectionElement } from "@/lib/types";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { seiya } from "@/store/seiya";
    import LinkButton from "@/components/UI/LinkButton.svelte";
    import Table from "@/components/UI/Table.svelte";
    import LinkToSidebar from "@/components/Work/LinkToSidebar.svelte";
    import { PlayStatus } from "@/lib/types";
    import { open } from "@tauri-apps/plugin-dialog";
    import { commandUpdateGameImage } from "@/lib/command";
    import ContextMenu from "@/components/UI/ContextMenu.svelte";
    import { fade } from "svelte/transition";

    export let work: Work;
    export let element: CollectionElement;

    $: seiyaUrlPromise = seiya.getUrl(work.name);

    $: summaryValue = [
        {
            label: "ブランド",
            value: work.brandName,
            component: LinkToSidebar,
        },
        { label: "発売日", value: work.sellday },
        { label: "平均プレイ時間", value: `${work.statistics.playTime}` },
        { label: "中央値", value: `${work.statistics.median}` },
        { label: "データ数", value: `${work.statistics.count}` },
    ];

    const getStatusLabel = (status: PlayStatus) => {
        switch (status) {
            case PlayStatus.Unplayed:
                return "未プレイ";
            case PlayStatus.Playing:
                return "プレイ中";
            case PlayStatus.Cleared:
                return "クリア済み";
            default:
                return "未プレイ";
        }
    };

    const getStatusColor = (status: PlayStatus) => {
        switch (status) {
            case PlayStatus.Playing:
                return "bg-accent-accent text-white";
            case PlayStatus.Cleared:
                return "bg-accent-success text-white";
            default:
                return "bg-bg-tertiary text-text-tertiary";
        }
    };

    let menu = {
        isOpen: false,
        x: 0,
        y: 0,
    };

    let isImageViewerOpen = false;

    const handleContextMenu = (e: MouseEvent) => {
        e.preventDefault();
        menu = {
            isOpen: true,
            x: e.clientX,
            y: e.clientY,
        };
    };

    const menuOptions = [
        {
            label: "サムネイルを変更",
            onSelect: async () => {
                const selected = await open({
                    multiple: false,
                    filters: [
                        { name: "Images", extensions: ["png", "jpg", "jpeg"] },
                    ],
                });
                if (selected && typeof selected.path === "string") {
                    await commandUpdateGameImage(
                        element.id,
                        "thumbnail",
                        selected.path,
                    );
                    window.location.reload();
                } else if (selected && typeof selected === "string") {
                    // Fallback if path is not present (depending on version)
                    await commandUpdateGameImage(
                        element.id,
                        "thumbnail",
                        selected,
                    );
                    window.location.reload();
                }
            },
        },
    ];
</script>

<div class="w-full min-h-full relative overflow-hidden">
    <!-- Ambient Background Layer -->
    {#if element.thumbnail}
        <div class="fixed inset-0 z-0 pointer-events-none">
            <img
                src={convertFileSrc(element.thumbnail)}
                alt=""
                class="w-full h-full object-cover blur-3xl opacity-30 scale-110"
            />
            <div class="absolute inset-0 bg-bg-primary/40"></div>
        </div>
    {/if}

    <!-- Cinematic Header -->
    <div
        class="relative w-full min-h-[60vh] group flex flex-col justify-end z-10"
        on:contextmenu={handleContextMenu}
    >
        {#if element.thumbnail}
            <!-- Header Background Image -->
            <img
                src={convertFileSrc(element.thumbnail)}
                alt={work.name}
                class="absolute inset-0 w-full h-full object-cover object-top transition-transform duration-700"
                style="mask-image: linear-gradient(to bottom, black 95%, transparent 100%); -webkit-mask-image: linear-gradient(to bottom, black 95%, transparent 100%);"
            />

            <!-- Gradients for Seamless Transition -->
            <!-- Top Gradient -->
            <div
                class="absolute inset-x-0 top-0 h-32 bg-gradient-to-b from-bg-primary/0 to-transparent"
            />
            <!-- Bottom Gradient - Fades to transparent to reveal ambient background, but darkens for text -->
            <div
                class="absolute inset-x-0 bottom-0 h-[50vh] bg-gradient-to-t from-bg-primary/90 via-bg-primary/30 to-transparent"
            />

            <!-- Clear image overlay (Floating) -->
            <div class="absolute right-12 top-12 hidden md:block z-20">
                <!-- svelte-ignore a11y-click-events-have-key-events -->
                <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                <img
                    src={convertFileSrc(element.thumbnail)}
                    alt={work.name}
                    class="w-auto h-auto max-h-[300px] max-w-[300px] rounded-lg shadow-2xl border border-white/10 cursor-pointer hover:scale-105 transition-transform duration-300"
                    on:click={() => (isImageViewerOpen = true)}
                />
            </div>
        {/if}

        <!-- Content Container -->
        <div
            class="relative z-20 w-full p-8 md:p-12 flex flex-col items-start gap-6 mt-auto"
        >
            <!-- Status Badge -->
            <div
                class="px-3 py-1 rounded-full text-xs font-bold tracking-wide uppercase {getStatusColor(
                    element.playStatus ?? PlayStatus.Unplayed,
                )} backdrop-blur-md shadow-lg border border-white/10"
            >
                {getStatusLabel(element.playStatus ?? PlayStatus.Unplayed)}
            </div>

            <!-- Title -->
            <h1
                class="text-5xl md:text-7xl font-bold text-white drop-shadow-2xl leading-tight max-w-5xl"
            >
                {work.name}
            </h1>

            <!-- Actions -->
            <div class="flex items-center gap-4 mt-2">
                {#await seiyaUrlPromise then seiyaUrl}
                    <Actions id={work.id} name={work.name} {seiyaUrl} />
                {/await}
            </div>
        </div>
    </div>

    <!-- Content Body -->
    <div
        class="relative z-10 p-8 md:p-12 grid grid-cols-1 lg:grid-cols-[2fr_1fr] gap-12 bg-gradient-to-b from-bg-primary/90 to-bg-primary"
    >
        <div class="space-y-8">
            <!-- External Links -->
            <div class="flex items-center gap-3 flex-wrap">
                <LinkButton
                    href={work.officialHomePage}
                    text="Official"
                    withIcon
                />
                <LinkButton
                    href="https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game={work.id}"
                    text="ErogameScape"
                    withIcon
                />
                {#await seiyaUrlPromise then url}
                    <LinkButton href={url} text="誠也の部屋" withIcon />
                {/await}
            </div>

            <Detail {work} />
        </div>

        <div class="space-y-6">
            <div class="p-6 rounded-xl glass-card">
                <h3 class="text-lg font-bold text-text-primary mb-4">
                    Information
                </h3>
                <Table title="" rows={summaryValue} />
            </div>
        </div>
    </div>
</div>

{#if menu.isOpen}
    <ContextMenu
        x={menu.x}
        y={menu.y}
        options={menuOptions}
        on:close={() => (menu.isOpen = false)}
    />
{/if}

{#if isImageViewerOpen && element.thumbnail}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
        class="fixed inset-0 z-[100] bg-black/90 flex items-center justify-center p-8 cursor-pointer"
        on:click={() => (isImageViewerOpen = false)}
        transition:fade={{ duration: 200 }}
    >
        <img
            src={convertFileSrc(element.thumbnail)}
            alt={work.name}
            class="max-w-full max-h-full object-contain rounded-lg shadow-2xl"
        />
    </div>
{/if}
