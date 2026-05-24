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
    class="mx-1 h-6 min-w-8 flex items-center justify-center rounded-full bg-[#0284c7] px-2 text-white shadow-sm transition-colors hover:bg-[#0369a1] outline-none border-none focus:outline-none focus-visible:outline-none"
  >
    <div class="i-material-symbols-system-update-alt-rounded text-[17px]" />
  </button>
{/if}
