<script lang="ts">
  import { onMount } from "svelte";
  import Work from "@/components/Work/Work.svelte";
  import { commandGetCollectionElement } from "@/lib/command";
  import { createEventDispatcher } from "svelte";

  export let params: { id: string };

  let elementPromise = commandGetCollectionElement(+params.id);

  // This function will be called by the child component to refetch data
  const refetch = () => {
    elementPromise = commandGetCollectionElement(+params.id);
  };
</script>

{#await elementPromise then element}
  <div class="w-full h-full">
    <!-- Pass the element and the refetch function to the child -->
    <Work bind:work={element} on:update={refetch} />
  </div>
{/await}
