<script lang="ts">
  import { appUpdate } from "@/store/update";
</script>

{#if $appUpdate.status === "available" && $appUpdate.update}
  <button
    data-update-badge
    type="button"
    tabindex="-1"
    title="更新があります"
    aria-label="更新があります"
    on:click={() => appUpdate.toggleDialog()}
    class="mx-2 h-6 min-w-10 flex items-center justify-center rounded-full bg-[#2563eb] px-3 text-white shadow-sm transition-colors hover:bg-[#1d4ed8] outline-none border-none focus:outline-none focus-visible:outline-none"
  >
    <div class="i-material-symbols-system-update-alt-rounded text-[17px]" />
  </button>
{/if}
