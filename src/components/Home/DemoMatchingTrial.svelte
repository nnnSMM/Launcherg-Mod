<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import {
    commandPreviewDemoGameMatching,
    type DemoGameMatchingPreview,
  } from "@/lib/command";
  import { showErrorToast } from "@/lib/toast";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";

  const labels = {
    title: "\u30d5\u30a9\u30eb\u30c0\u7d10\u3065\u3051\u3092\u8a66\u3059",
    description:
      "\u6240\u6301\u30d5\u30a9\u30eb\u30c0\u3092\u9078\u3076\u3068\u3001\u5b9f\u884c\u30d5\u30a1\u30a4\u30eb\u304b\u3089\u3069\u306e\u30b2\u30fc\u30e0\u306b\u7d10\u3065\u304f\u304b\u3092\u767b\u9332\u305b\u305a\u306b\u5224\u5b9a\u3057\u307e\u3059\u3002",
    chooseFolder: "\u30d5\u30a9\u30eb\u30c0\u3092\u9078\u629e",
    chooseMore: "\u5225\u306e\u30d5\u30a9\u30eb\u30c0\u3082\u9078\u629e",
    clear: "\u7d50\u679c\u3092\u30af\u30ea\u30a2",
    processing: "\u30d5\u30a9\u30eb\u30c0\u3092\u8aad\u307f\u53d6\u308a\u4e2d",
    processingDetail:
      "\u81ea\u52d5\u8ffd\u52a0\u3068\u540c\u3058\u3088\u3046\u306b exe / lnk \u3092\u63a2\u3057\u3001\u4f5c\u54c1\u5019\u88dc\u3068\u7167\u5408\u3057\u3066\u3044\u307e\u3059\u3002",
    processedFiles: "\u51e6\u7406\u3057\u305f\u30d5\u30a1\u30a4\u30eb",
    result: "\u5224\u5b9a\u7d50\u679c",
    selectedFolders: "\u9078\u629e\u3057\u305f\u30d5\u30a9\u30eb\u30c0",
    matched: "\u7d10\u3065\u3044\u305f\u30b2\u30fc\u30e0",
    unmatched: "\u7d10\u3065\u3051\u3067\u304d\u306a\u304b\u3063\u305f\u30d1\u30b9",
    linkedGames: "\u9ad8\u4fe1\u983c\u3067\u7d10\u3065\u3044\u305f\u30b2\u30fc\u30e0",
    highConfidence: "\u9ad8\u4fe1\u983c",
    needsReview: "\u78ba\u8a8d\u304c\u5fc5\u8981",
    noCandidate: "\u5019\u88dc\u306a\u3057",
    noFiles:
      "\u5bfe\u8c61\u306b\u306a\u308b exe / lnk \u304c\u898b\u3064\u304b\u308a\u307e\u305b\u3093\u3067\u3057\u305f\u3002",
    readError:
      "\u30d5\u30a9\u30eb\u30c0\u306e\u8aad\u307f\u53d6\u308a\u306b\u5931\u6557\u3057\u307e\u3057\u305f\u3002\u5225\u306e\u30d5\u30a9\u30eb\u30c0\u3067\u8a66\u3057\u3066\u304f\u3060\u3055\u3044\u3002",
    canceled:
      "\u30d5\u30a9\u30eb\u30c0\u9078\u629e\u304c\u30ad\u30e3\u30f3\u30bb\u30eb\u3055\u308c\u307e\u3057\u305f\u3002",
    items: "\u4ef6",
  };

  let selectedPaths: string[] = [];
  let isLoading = false;
  let processFileNums = 0;
  let processedFileNums = 0;
  let preview: DemoGameMatchingPreview | null = null;

  const filename = (path: string) => path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;

  const runPreview = async (paths: string[]) => {
    if (!paths.length) {
      preview = null;
      return;
    }
    processFileNums = 0;
    processedFileNums = 0;
    isLoading = true;
    try {
      preview = await commandPreviewDemoGameMatching(paths);
    } catch (e) {
      console.error(e);
      showErrorToast(labels.readError);
    } finally {
      isLoading = false;
    }
  };

  const selectFolder = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      filters: [],
    });
    if (typeof selected !== "string") {
      showErrorToast(labels.canceled);
      return;
    }
    selectedPaths = [...selectedPaths, selected];
    await runPreview(selectedPaths);
  };

  const clear = () => {
    selectedPaths = [];
    preview = null;
    processFileNums = 0;
    processedFileNums = 0;
  };

  onMount(async () => {
    const unlistenProgressLive = await listen<{ max: number | null }>(
      "progresslive",
      (event) => {
        if (typeof event.payload?.max === "number") {
          processFileNums = event.payload.max;
          processedFileNums = 0;
        } else {
          processedFileNums = processedFileNums + 1;
        }
      },
    );
    return () => {
      unlistenProgressLive();
    };
  });

  $: matchedResults = preview?.results.filter((result) => result.matched) ?? [];
  $: unmatchedResults = preview?.results.filter((result) => !result.matched) ?? [];
</script>

<section class="rounded-lg border border-ui-border bg-bg-secondary/80 p-5 shadow-sm">
  <div class="flex flex-col gap-4 lg:flex-row lg:items-start">
    <div class="min-w-0 flex-1 space-y-4">
      <div class="flex items-center gap-3">
        <div class="h-9 w-9 flex shrink-0 items-center justify-center rounded-md bg-accent-accent/15">
          <div class="i-material-symbols:folder-search-rounded h-5 w-5 color-accent-accent" />
        </div>
        <div class="min-w-0">
          <h2 class="text-h3 font-bold text-text-primary">{labels.title}</h2>
          <p class="mt-1 text-body2 text-text-tertiary">{labels.description}</p>
        </div>
      </div>

      <div class="flex flex-wrap gap-2">
        <Button
          text={selectedPaths.length ? labels.chooseMore : labels.chooseFolder}
          leftIcon="i-material-symbols:create-new-folder-rounded"
          variant="accent"
          disabled={isLoading}
          on:click={selectFolder}
        />
        {#if selectedPaths.length}
          <Button
            text={labels.clear}
            leftIcon="i-iconoir-cancel"
            borderless
            disabled={isLoading}
            on:click={clear}
          />
        {/if}
      </div>

      {#if selectedPaths.length}
        <div class="rounded-md border border-ui-border bg-bg-primary/70 p-3">
          <div class="mb-2 text-caption font-bold text-text-tertiary">
            {labels.selectedFolders}
          </div>
          <div class="space-y-1">
            {#each selectedPaths as path}
              <div class="truncate text-body2 text-text-secondary">{path}</div>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    {#if preview}
      <div class="w-full rounded-md border border-ui-border bg-bg-primary/70 p-4 lg:w-64">
        <div class="text-body2 text-text-tertiary">{labels.result}</div>
        <div class="mt-1 text-3xl font-bold text-text-primary">
          {preview.matchedCount} {labels.items}
        </div>
        <div class="mt-2 text-body2 text-text-secondary">
          {labels.linkedGames}
        </div>
      </div>
    {/if}
  </div>

  {#if isLoading}
    <div class="mt-5 flex items-center gap-4 rounded-md border border-ui-border bg-bg-primary/70 p-4">
      <div class="h-12 w-12 shrink-0 rounded-full border-6px border-solid border-bg-tertiary border-t-accent-accent animate-spin" />
      <div class="min-w-0">
        <div class="text-body font-bold text-text-primary">{labels.processing}</div>
        <div class="mt-1 text-body2 text-text-tertiary">{labels.processingDetail}</div>
        {#if processFileNums}
          <div class="mt-2 text-body2 font-medium text-text-primary">
            {labels.processedFiles}: {processedFileNums}/{processFileNums}
          </div>
        {/if}
      </div>
    </div>
  {:else if preview}
    <div class="mt-5 space-y-4">
      {#if preview.results.length === 0}
        <div class="rounded-md border border-ui-border bg-bg-primary/70 p-4 text-body2 text-text-tertiary">
          {labels.noFiles}
        </div>
      {:else}
        <div class="space-y-2">
          <div class="text-body font-bold text-text-primary">{labels.matched}</div>
          {#if matchedResults.length}
            {#each matchedResults as result}
              <div class="grid gap-3 rounded-md border border-ui-border bg-bg-primary/70 p-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)]">
                <div class="min-w-0">
                  <div class="truncate text-body2 font-bold text-text-primary">
                    {result.matched?.gamename}
                  </div>
                  <div class="mt-1 inline-flex items-center gap-1 rounded bg-accent-success/15 px-2 py-0.5 text-caption text-accent-success">
                    <div class="i-material-symbols:check-circle-rounded h-3.5 w-3.5" />
                    {labels.highConfidence}
                  </div>
                </div>
                <div class="min-w-0">
                  <div class="truncate text-body2 text-text-secondary">
                    {filename(result.path)}
                  </div>
                  <div class="mt-1 truncate text-caption text-text-tertiary">
                    {result.path}
                  </div>
                </div>
              </div>
            {/each}
          {:else}
            <div class="rounded-md border border-ui-border bg-bg-primary/70 p-3 text-body2 text-text-tertiary">
              {labels.noCandidate}
            </div>
          {/if}
        </div>

        {#if unmatchedResults.length}
          <div class="space-y-2">
            <div class="text-body font-bold text-text-primary">{labels.unmatched}</div>
            {#each unmatchedResults as result}
              <div class="grid gap-3 rounded-md border border-ui-border bg-bg-primary/70 p-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)]">
                <div class="min-w-0">
                  <div class="text-body2 text-text-secondary">
                    {result.candidates.length ? labels.needsReview : labels.noCandidate}
                  </div>
                  {#if result.candidates.length}
                    <div class="mt-1 truncate text-caption text-text-tertiary">
                      {result.candidates[0][1]}
                    </div>
                  {/if}
                </div>
                <div class="min-w-0">
                  <div class="truncate text-body2 text-text-secondary">
                    {filename(result.path)}
                  </div>
                  <div class="mt-1 truncate text-caption text-text-tertiary">
                    {result.path}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</section>
