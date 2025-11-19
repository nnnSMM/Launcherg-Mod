<script lang="ts">
  import WorkLayout from "@/components/Work/WorkLayout.svelte";
  import type { Work } from "@/lib/types";
  import { onMount } from "svelte";
  import { backgroundState } from "@/store/background";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import SimpleBar from "simplebar";

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

  const simplebar = (node: HTMLElement) => {
    new SimpleBar(node);
  };
</script>

<div use:simplebar class="h-full w-full overflow-x-hidden overflow-y-auto">
  <div class="w-full min-h-0 flex justify-center">
    {#key work.imgUrl}
      <WorkLayout {work} {element} />
    {/key}
  </div>
</div>
