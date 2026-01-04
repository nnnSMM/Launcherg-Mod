<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { CollectionElement } from "@/lib/types";

    export let games: CollectionElement[] = [];
    export let selectedGameId: number | null;

    const dispatch = createEventDispatcher();
    let open = false;
    let searchQuery = "";

    $: filteredGames = games.filter((g) =>
        g.gamename.toLowerCase().includes(searchQuery.toLowerCase()),
    );

    const select = (id: number | null) => {
        dispatch("select", id);
        open = false;
        searchQuery = "";
    };

    function clickOutside(node: Node) {
        const handleClick = (event: MouseEvent) => {
            if (
                node &&
                !node.contains(event.target as Node) &&
                !event.defaultPrevented
            ) {
                open = false;
            }
        };

        document.addEventListener("click", handleClick, true);

        return {
            destroy() {
                document.removeEventListener("click", handleClick, true);
            },
        };
    }

    $: selectedGameName = selectedGameId
        ? games.find((g) => g.id === selectedGameId)?.gamename
        : "All Games";
</script>

<div class="relative z-50" use:clickOutside>
    <button
        class="flex items-center space-x-2 px-3 py-2 rounded text-sm min-w-[200px] justify-between transition-colors border"
        style="background-color: #2d2d2d; border-color: #404040;"
        on:click|stopPropagation={() => (open = !open)}
    >
        <span class="truncate max-w-[180px] font-medium" style="color: #e0e0e0;"
            >{selectedGameName}</span
        >
        <span
            class="i-material-symbols-arrow-drop-down text-lg transition-transform {open
                ? 'rotate-180'
                : ''}"
            style="color: #a0a0a0;"
        />
    </button>

    {#if open}
        <div
            class="absolute top-full left-0 mt-1 w-72 rounded-lg shadow-xl overflow-hidden flex flex-col max-h-[500px] border"
            style="background-color: #1e1e1e; border-color: #404040;"
        >
            <div
                class="p-2 border-b sticky top-0 z-10"
                style="background-color: #1e1e1e; border-color: #404040;"
            >
                <input
                    type="text"
                    placeholder="Search games..."
                    bind:value={searchQuery}
                    class="w-full rounded px-3 py-1.5 text-sm focus:outline-none transition-colors border"
                    style="background-color: #2d2d2d; border-color: #404040; color: #e0e0e0;"
                />
            </div>

            <div class="overflow-y-auto flex-1 p-1">
                <button
                    class="w-full text-left px-3 py-2 rounded text-sm transition-colors flex items-center mb-1"
                    style={selectedGameId === null
                        ? "background-color: #3b82f6; color: white; font-weight: bold;"
                        : "background-color: transparent; color: #e0e0e0;"}
                    on:click={() => select(null)}
                    on:mouseenter={(e) => {
                        if (selectedGameId !== null) {
                            e.currentTarget.style.backgroundColor = "#3b82f6";
                            e.currentTarget.style.color = "white";
                        }
                    }}
                    on:mouseleave={(e) => {
                        if (selectedGameId !== null) {
                            e.currentTarget.style.backgroundColor =
                                "transparent";
                            e.currentTarget.style.color = "#e0e0e0";
                        }
                    }}
                >
                    <span class="i-material-symbols-apps mr-2" />
                    All Games
                </button>

                {#if filteredGames.length === 0}
                    <div
                        class="p-4 text-center text-sm"
                        style="color: #808080;"
                    >
                        No games found
                    </div>
                {/if}

                {#each filteredGames as game}
                    <button
                        class="w-full text-left px-3 py-2 rounded text-sm transition-colors truncate"
                        style={selectedGameId === game.id
                            ? "background-color: #3b82f6; color: white; font-weight: bold;"
                            : "background-color: transparent; color: #e0e0e0;"}
                        on:click={() => select(game.id)}
                        on:mouseenter={(e) => {
                            if (selectedGameId !== game.id) {
                                e.currentTarget.style.backgroundColor =
                                    "#3b82f6";
                                e.currentTarget.style.color = "white";
                            }
                        }}
                        on:mouseleave={(e) => {
                            if (selectedGameId !== game.id) {
                                e.currentTarget.style.backgroundColor =
                                    "transparent";
                                e.currentTarget.style.color = "#e0e0e0";
                            }
                        }}
                        title={game.gamename}
                    >
                        {game.gamename}
                    </button>
                {/each}
            </div>
        </div>
    {/if}
</div>
