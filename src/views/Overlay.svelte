<script lang="ts">
    import { commandTogglePauseTracking } from "@/lib/command";
    import { onMount } from "svelte";

    const resume = async () => {
        await commandTogglePauseTracking();
    };

    onMount(() => {
        // Ensure focus is kept or handled if needed
    });
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
    class="fixed inset-0 z-[9999] bg-black/60 flex items-center justify-center cursor-pointer backdrop-blur-sm"
    on:click={resume}
>
    <div class="text-center text-white pointer-events-none select-none">
        <div class="text-6xl font-bold mb-4 drop-shadow-lg">PAUSED</div>
        <div class="text-xl opacity-80 drop-shadow-md">
            Click anywhere to resume
        </div>
    </div>
</div>
