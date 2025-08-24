<script lang="ts">
  import { showSidebar } from "@/store/showSidebar";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
</script>

<div class="flex flex-col h-full w-full items-center gap-4 py-4">
  <div
    class="p-1 rounded cursor-pointer hover:bg-white/10"
    on:click={() => showSidebar.set(true)}
    role="button"
    tabindex="0"
    on:keydown={(e) => e.key === "Enter" && showSidebar.set(true)}
  >
    <div
      class="i-material-symbols-left-panel-close-outline w-6 h-6 color-text-primary rotate-180"
    />
  </div>
</div>
