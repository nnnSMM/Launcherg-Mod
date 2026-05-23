<script lang="ts">
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatcher = createEventDispatcher<{
    close: {};
    startAuto: {};
    startManual: {};
  }>();

  const labels = {
    auto: "\u30d5\u30a9\u30eb\u30c0\u304b\u3089\u81ea\u52d5\u8ffd\u52a0",
    manual: "\u624b\u52d5\u3067\u8ffd\u52a0",
  };
</script>

<div class="py-1">
  <OptionButton
    text={labels.auto}
    on:click={() => {
      dispatcher("close");
      dispatcher("startAuto");
    }}
  />
  <OptionButton
    text={labels.manual}
    on:click={() => {
      dispatcher("close");
      dispatcher("startManual");
    }}
  />
</div>
