<script lang="ts">
  import Icon from "/icon.png";
  import { link } from "svelte-spa-router";
  import { showSidebar } from "@/store/showSidebar";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import { openSettingsTab } from "@/store/tabs";

</script>

<div class="w-full flex items-center px-2 pt-4">
  <a href="/" use:link class="mr-auto">
    <div class="flex items-center gap-2 w-full">
      <img src={Icon} alt="launcherg icon" class="h-8" />
      <div class="font-logo text-(h3 text-primary)">Launcherg</div>
    </div>
  </a>

  <ButtonBase
    on:click={openSettingsTab}
    appendClass="p-1 bg-transparent border-0px"
    tooltip={{
      content: "一括編集",
      placement: "bottom",
      theme: "default",
      delay: 1000,
    }}
  >
    <div class="i-material-symbols-label-outline-rounded w-6 h-6 color-text-primary" />
  </ButtonBase>

  <ButtonBase
    on:click={() => showSidebar.set(false)}
    appendClass="ml-1 border-0px p-1 bg-transparent"
    tooltip={{
      content: "サイドバーを閉じる",
      placement: "bottom",
      theme: "default",
      delay: 1000,
    }}
  >
    <div
      class="i-material-symbols-left-panel-close-outline w-6 h-6 color-text-primary"
    />
  </ButtonBase>
</div>
