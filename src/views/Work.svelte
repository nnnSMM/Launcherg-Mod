<script lang="ts">
  import { onMount } from "svelte";
  import { works } from "@/store/works";
  import Work from "@/components/Work/Work.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { commandGetCollectionElement } from "@/lib/command";
  import type { CollectionElement, Work as WorkType } from "@/lib/types";
  import { listen } from "@tauri-apps/api/event";

  export let params: { id: string };

  let currentWork: WorkType | null = null;
  let currentElement: CollectionElement | null = null;
  let errorMsg: string | null = null;

  const loadData = async (id: number) => {
    try {
      errorMsg = null;
      const w = await works.get(id);
      if (!w) throw new Error("Work not found");
      const e = await commandGetCollectionElement(w.id);
      currentWork = w;
      currentElement = e;
    } catch (e: any) {
      errorMsg = e.message;
    }
  };

  $: if (params.id) {
    currentWork = null;
    currentElement = null;
    loadData(+params.id);
  }

  onMount(() => {
    sidebarCollectionElements.refetch();

    const unlisten = listen<number>("collection-element-updated", (event) => {
      if (event.payload === +params.id) {
        // UIの再構築を防ぐため、プロパティの静かな更新のみ行う
        (async () => {
          if (currentWork) {
            const e = await commandGetCollectionElement(currentWork.id);
            currentElement = e;
          }
        })();
      }
      // サイドバーのプレイ時間等も最新化
      sidebarCollectionElements.refetch();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

{#if errorMsg}
  <div class="p-4 text-text-error">
    Error loading game data: {errorMsg}
  </div>
{:else if currentWork && currentElement}
  <div class="w-full h-full">
    <Work work={currentWork} element={currentElement} />
  </div>
{/if}
