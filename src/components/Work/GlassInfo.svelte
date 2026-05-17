<script lang="ts">
    import Actions from "@/components/Work/Actions.svelte";
    import { seiya } from "@/store/seiya";
    import Info from "@/components/Work/Info.svelte";
    import type { Work, CollectionElement } from "@/lib/types";

    export let work: Work;
    export let element: CollectionElement;
</script>

<div
    class="relative z-20 bg-bg-primary/30 border-t border-border-primary p-6 lg:p-8 shadow-2xl space-y-6"
    style="backdrop-filter: blur(8px);"
>
    {#await seiya.getUrl(work.name)}
        <Actions id={work.id} name={element.gamename} seiyaUrl={""} />
    {:then seiyaUrl}
        <Actions id={work.id} name={element.gamename} {seiyaUrl} />
    {/await}

    <div class="border-t border-border-primary pt-6">
        <Info {work} />
    </div>
</div>
