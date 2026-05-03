<script lang="ts">
  import { showSidebar } from "@/store/showSidebar";
</script>

<div class="mt-4 w-full px-2 flex items-center min-w-0">
  <div class="text-(text-primary body) font-bold pl-2 mr-auto truncate min-w-0">
    登録したゲーム
  </div>
  <div
    class="ml-1 p-1 rounded cursor-pointer hover:bg-white/10"
    on:click={() => showSidebar.set(false)}
    role="button"
    tabindex="-1"
    on:keydown={(e) => e.key === "Enter" && showSidebar.set(false)}
  >
    <div
      class="i-material-symbols-left-panel-close-outline w-6 h-6 color-text-primary"
    />
  </div>
</div>
