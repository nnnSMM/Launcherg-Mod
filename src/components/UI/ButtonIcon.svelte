<script lang="ts">
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import { type Props as TippyOption } from "tippy.js";

  export let icon: string;
  export let tooltip: TippyOption | undefined = undefined;
  export let appendClass = "";
  export let borderless = false;
</script>

<ButtonBase
  appendClass={`h-8 w-8 flex items-center justify-center ${appendClass}`}
  {tooltip}
  {borderless}
>
  <div class="color-ui-tertiary w-5 h-5 {icon}" />
</ButtonBase>
