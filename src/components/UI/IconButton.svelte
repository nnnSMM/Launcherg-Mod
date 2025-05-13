<script lang="ts">
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import type { Props as TippyOption } from "tippy.js"; // Tippyの型をインポート

  export let icon: string;
  export let tooltip: Partial<TippyOption> | undefined = undefined; // tooltip プロパティを追加
  export let appendClass: string = ""; // IconButtonに直接クラスを渡せるようにする
</script>

<ButtonBase
  appendClass="h-8 w-8 p-2 {appendClass}"
  on:click
  {tooltip}
>
  <div class={`color-ui-tertiary w-4 h-4 ${icon}`} />
</ButtonBase>
