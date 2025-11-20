<script lang="ts">
    import Actions from "@/components/Work/Actions.svelte";
    import { seiya } from "@/store/seiya";
    import Info from "@/components/Work/Info.svelte";
    import type { Work, CollectionElement } from "@/lib/types";

    export let work: Work;
    export let element: CollectionElement;
</script>

<div
    class="backdrop-blur-md bg-black/10 border-t border-white/10 p-12 shadow-2xl space-y-6 -mt-8"
>
    {#await seiya.getUrl(work.name) then seiyaUrl}
        <Actions id={work.id} name={element.gamename} {seiyaUrl} />
    {/await}

    <div class="border-t border-white/10 pt-6">
        <Info {work} />
    </div>
</div>
