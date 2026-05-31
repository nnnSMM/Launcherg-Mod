<script lang="ts">
  export let totalCount = 0;
  export let shownCount = 0;

  const labels = {
    library: "\u30e9\u30a4\u30d6\u30e9\u30ea",
    shown: "\u8868\u793a",
    registered: "\u767b\u9332",
  };
</script>

<div class="w-full px-3 pt-3">
  <div
    class="flex items-center gap-3 rounded-lg border border-border-primary bg-bg-primary/30 px-3 py-2"
  >
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2 min-w-0">
        <div
          class="i-material-symbols:video-library-outline-rounded h-5 w-5 shrink-0 color-accent-accent"
        />
        <div class="truncate text-sm font-semibold text-text-primary">
          {labels.library}
        </div>
      </div>
      <div class="mt-0.5 flex items-center gap-2 text-[11px] text-text-tertiary">
        <span>{labels.shown} {shownCount}</span>
        <span class="h-1 w-1 rounded-full bg-border-primary" />
        <span>{labels.registered} {totalCount}</span>
      </div>
    </div>
  </div>
</div>
