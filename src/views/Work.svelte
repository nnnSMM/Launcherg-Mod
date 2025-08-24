<script lang="ts">
  import { onMount } from "svelte"; // 1. onMount をインポート
  import { works } from "@/store/works";
  import Work from "@/components/Work/Work.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements"; // 2. sidebarCollectionElements をインポート

  export let params: { id: string };

  // 3. ページが表示された時に、全要素の最新情報を再取得する処理を追加
  onMount(() => {
    sidebarCollectionElements.refetch();
  });

  $: workPromise = works.get(+params.id);
</script>

{#await workPromise then work}
  <div class="w-full h-full">
    <Work {work} />
  </div>
{/await}
