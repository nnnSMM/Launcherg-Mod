<script lang="ts">
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatcher = createEventDispatcher<{
    playAdmin: {};
    play: {};
    close: {};
  }>();
</script>

<div class="py-1">
  <OptionButton
    text="管理者権限で起動"
    on:click={() => {
      dispatcher("close");
      dispatcher("playAdmin");
    }}
  />
  <OptionButton
    text="現在のユーザーで起動"
    on:click={() => {
      dispatcher("close");
      dispatcher("play");
    }}
  />
</div>
