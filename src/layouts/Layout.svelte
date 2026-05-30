<script lang="ts">
  import Sidebar from "@/components/Sidebar/Sidebar.svelte";
  import ATabList from "@/components/Tab/ATabList.svelte";
</script>

<div class="grid grid-cols-[min-content_1fr] w-full h-full bg-transparent">
  <Sidebar />
  <div
    class="h-full min-w-0 min-h-0 flex flex-col overflow-hidden"
  >
    <ATabList />
    <div class="relative flex-1 min-h-0 min-w-0 w-full">
      <slot />
    </div>
  </div>
</div>
