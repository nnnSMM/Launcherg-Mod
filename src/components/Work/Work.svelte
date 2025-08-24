<script lang="ts">
  import WorkLayout from "@/components/Work/WorkLayout.svelte";
  import type { Work } from "@/lib/types";
  import { onMount, onDestroy } from "svelte";
  import { backgroundState } from "@/store/background";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";

  export let work: Work;
  export let element: CollectionElement;

  onMount(() => {
    if (element && element.thumbnail) {
      backgroundState.set({
        imageUrl: convertFileSrc(element.thumbnail),
        opacity: 0.2,
      });
    }
  });

  onDestroy(() => {
    backgroundState.set({
      imageUrl: null,
      opacity: 0,
    });
  });
</script>

<div class="h-full w-full overflow-x-hidden overflow-y-auto">
  <div class="w-full min-h-0 flex justify-center">
    {#key work.imgUrl}
      <WorkLayout {work} {element} />
    {/key}
  </div>
</div>
