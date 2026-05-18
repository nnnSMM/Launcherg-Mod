<script lang="ts">
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatcher = createEventDispatcher<{
    close: {};
    startAuto: {};
    startManual: {};
  }>();

  const labels = {
    title: "\u30b2\u30fc\u30e0\u8ffd\u52a0\u65b9\u6cd5\u3092\u9078\u629e",
    auto: "\u30d5\u30a9\u30eb\u30c0\u304b\u3089\u81ea\u52d5\u8ffd\u52a0",
    manual: "\u624b\u52d5\u3067\u8ffd\u52a0",
  };
</script>

<div>
  <div
    class="font-bold text-text-primary text-body3 p-l-4 p-r-2 p-y-2 flex items-center gap-4"
  >
    <div class="whitespace-nowrap">{labels.title}</div>
    <button
      on:click={() => dispatcher("close")}
      class="ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all"
    />
  </div>
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
