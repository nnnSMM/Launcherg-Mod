<script lang="ts">
  import { showSidebar } from "@/store/showSidebar";

</script>

<div class="w-full flex items-center px-2 pt-4">
  <div class="mr-auto"></div>

  <div
    class="ml-1 p-1 rounded cursor-pointer hover:bg-white/10"
    on:click={() => showSidebar.set(false)}
    role="button"
    tabindex="0"
    on:keydown={(e) => e.key === "Enter" && showSidebar.set(false)}
  >
    <div
      class="i-material-symbols-left-panel-close-outline w-6 h-6 color-text-primary"
    />
  </div>
</div>
