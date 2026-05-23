<script lang="ts">
  import { SORT_LABELS, type SortOrder } from "@/components/Sidebar/sort";
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  export let value: SortOrder;
  const dispatcher = createEventDispatcher<{ close: {} }>();

  const sortOrders: SortOrder[] = [
    "gamename-asc",
    "gamename-desc",
    "sellyear-desc",
    "install-desc",
    "last_play-desc",
    "registered-desc",
    "brandname-asc",
    "brandname-desc",
  ];
</script>

<div class="py-1">
  {#each sortOrders as sortOrder (sortOrder)}
    <OptionButton
      on:click={() => {
        value = sortOrder;
        dispatcher("close");
      }}
      selected={value === sortOrder}
      text={SORT_LABELS[sortOrder]}
      showIcon
    />
  {/each}
</div>
