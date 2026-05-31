<script lang="ts">
  import { showSidebar } from "@/store/showSidebar";

  const labels = {
    open: "\u30b5\u30a4\u30c9\u30d0\u30fc\u3092\u958b\u304f",
    library: "\u30e9\u30a4\u30d6\u30e9\u30ea",
  };
</script>

<div class="flex h-full w-full flex-col items-center gap-3 py-3">
  <button
    type="button"
    class="grid h-8 w-8 place-items-center rounded-md bg-bg-primary/30 text-text-secondary transition-colors hover:bg-white/10 hover:text-text-primary focus-visible:ring-2 focus-visible:ring-accent-accent"
    on:click={() => showSidebar.set(true)}
    aria-label={labels.open}
    title={labels.open}
  >
    <div
      class="i-material-symbols:left-panel-open-outline h-5 w-5"
    />
  </button>
  <div class="h-px w-6 bg-border-primary" />
  <div
    class="grid h-8 w-8 place-items-center rounded-md text-text-tertiary"
    aria-label={labels.library}
    title={labels.library}
  >
    <div
      class="i-material-symbols:video-library-outline-rounded h-5 w-5"
    />
  </div>
</div>
