<script lang="ts">
  import LinkButton from "@/components/UI/LinkButton.svelte";
  import Actions from "@/components/Work/Actions.svelte";
  import LinkToSidebar from "@/components/Work/LinkToSidebar.svelte";
  import type { Work } from "@/lib/types";
  import { seiya } from "@/store/seiya";

  export let work: Work;

  $: seiyaUrlPromise = seiya.getUrl(work.name);
</script>

<div class="space-y-6 max-w-full">
  <div class="space-y-4">
    <div class="text-h1 font-bold">{work.name}</div>
    <div class="text-body2 text-secondary">
      <LinkToSidebar value={work.brandName} />
    </div>
  </div>

  {#await seiyaUrlPromise then seiyaUrl}
    <Actions id={work.id} name={work.name} {seiyaUrl} />
  {/await}

  <div class="flex items-center gap-x-4">
    <LinkButton href={work.officialHomePage} text="Official" withIcon />
    <LinkButton
      href="https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game={work.id}"
      text="ErogameScape"
      withIcon
    />
    {#await seiyaUrlPromise then url}
      <LinkButton href={url} text="誠也の部屋" withIcon />
    {/await}
  </div>

  <div class="space-y-4">
    <div class="text-h4 font-bold">Summary</div>
    <div class="grid grid-cols-2 gap-x-4 gap-y-2 text-body2">
      <div class="text-tertiary">発売日</div>
      <div class="text-secondary">{work.sellday}</div>
      <div class="text-tertiary">平均プレイ時間</div>
      <div class="text-secondary">{work.statistics.playTime}</div>
      <div class="text-tertiary">中央値</div>
      <div class="text-secondary">{work.statistics.median}</div>
      <div class="text-tertiary">データ数</div>
      <div class="text-secondary">{work.statistics.count}</div>
    </div>
  </div>
</div>
