<script lang="ts">
  import WorkLayout from "@/components/Work/WorkLayout.svelte";
  import type { CollectionElement } from "@/lib/types";
  import { createEventDispatcher } from "svelte";

  export let work: CollectionElement;
  const dispatcher = createEventDispatcher();
</script>

<div class="h-full w-full overflow-x-hidden overflow-y-auto">
  <div class="w-full min-h-0 flex justify-center">
    {#key work.updatedAt}
      <WorkLayout work={work} on:update={() => dispatcher("update")} />
    {/key}
  </div>
</div>
