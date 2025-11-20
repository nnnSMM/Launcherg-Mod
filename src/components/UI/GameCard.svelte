<script lang="ts">
    import type { CollectionElement } from "@/lib/types";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { link } from "svelte-spa-router";
    import { formatLastPlayed, formatPlayTime } from "@/lib/utils";

    export let collectionElement: CollectionElement;

    $: src = `${convertFileSrc(collectionElement.thumbnail)}?v=${
        collectionElement.updatedAt
    }`;
</script>

<a
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    use:link
    class="block w-full h-full relative group overflow-hidden rounded-xl shadow-lg transition-all duration-300 hover:shadow-xl hover:-translate-y-1"
>
    <div class="w-full h-full bg-bg-secondary">
        {#if collectionElement.thumbnail}
            <img
                {src}
                alt={collectionElement.gamename}
                class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                loading="lazy"
            />
        {:else}
            <div
                class="w-full h-full flex items-center justify-center text-text-tertiary p-4 text-center"
            >
                {collectionElement.gamename}
            </div>
        {/if}
    </div>

    <!-- Overlay Gradient -->
    <div
        class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"
    />

    <!-- Title and Info on Hover -->
    <div
        class="absolute bottom-0 left-0 right-0 p-4 translate-y-full group-hover:translate-y-0 transition-transform duration-300 flex flex-col gap-1"
    >
        <div class="text-white font-bold text-sm line-clamp-2 drop-shadow-md">
            {collectionElement.gamename}
        </div>

        <div
            class="flex flex-col gap-0.5 text-xs text-gray-300 font-medium drop-shadow-md opacity-0 group-hover:opacity-100 transition-opacity duration-300 delay-100"
        >
            {#if collectionElement.lastPlayAt}
                <div>
                    最終プレイ: {formatLastPlayed(collectionElement.lastPlayAt)}
                </div>
            {/if}
            {#if collectionElement.totalPlayTimeSeconds > 0}
                <div>
                    プレイ時間: {formatPlayTime(
                        collectionElement.totalPlayTimeSeconds,
                    )}
                </div>
            {/if}
        </div>
    </div>

    <!-- Status Badge - Show only on hover -->
    {#if collectionElement.playStatus === 2}
        <div
            class="absolute top-2 right-2 px-2 py-1 rounded bg-accent-success/90 text-white text-xs font-bold opacity-0 group-hover:opacity-100 transition-opacity"
        >
            CLEAR
        </div>
    {/if}
</a>
