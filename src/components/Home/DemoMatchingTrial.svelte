<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import InputPath from "@/components/UI/InputPath.svelte";
  import {
    commandPreviewDemoGameMatching,
    type DemoGameMatchingPreview,
  } from "@/lib/command";
  import { showErrorToast } from "@/lib/toast";

  type PathInput = {
    id: number;
    path: string;
  };

  let paths: PathInput[] = [{ id: Date.now(), path: "" }];
  let isLoading = false;
  let preview: DemoGameMatchingPreview | null = null;

  const nonEmptyPaths = () =>
    paths.map((value) => value.path.trim()).filter((path) => path.length > 0);

  const updatePath = (index: number, value: string) => {
    paths = paths.map((path, i) => (i === index ? { ...path, path: value } : path));
  };

  const addPath = () => {
    if (paths.at(-1)?.path.trim() === "") {
      return;
    }
    paths = [...paths, { id: Date.now(), path: "" }];
  };

  const removePath = (index: number) => {
    paths = paths.filter((_, i) => i !== index);
    if (!paths.length) {
      paths = [{ id: Date.now(), path: "" }];
    }
  };

  const filename = (path: string) => path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;

  const runPreview = async () => {
    const selectedPaths = nonEmptyPaths();
    if (!selectedPaths.length) {
      return;
    }
    isLoading = true;
    try {
      preview = await commandPreviewDemoGameMatching(selectedPaths);
    } catch (e) {
      console.error(e);
      showErrorToast("フォルダの読み取りに失敗しました。別のフォルダで試してください。");
    } finally {
      isLoading = false;
    }
  };

  $: matchRate =
    preview && preview.scannedFileCount > 0
      ? Math.round((preview.matchedCount / preview.scannedFileCount) * 100)
      : 0;
</script>

<section class="rounded-lg border border-ui-border bg-bg-secondary/80 p-5 shadow-sm">
  <div class="flex flex-col gap-4 lg:flex-row lg:items-start">
    <div class="min-w-0 flex-1 space-y-3">
      <div class="flex items-center gap-3">
        <div class="h-9 w-9 flex shrink-0 items-center justify-center rounded-md bg-accent-accent/15">
          <div class="i-material-symbols:folder-search-rounded h-5 w-5 color-accent-accent" />
        </div>
        <div class="min-w-0">
          <h2 class="text-h3 font-bold text-text-primary">フォルダ紐づけを試す</h2>
          <p class="mt-1 text-body2 text-text-tertiary">
            選択したフォルダ内の実行ファイルから、どのゲームとして判定されるかだけを確認します。demo ではゲーム登録は行いません。
          </p>
        </div>
      </div>

      <div class="space-y-2">
        {#each paths as item, i (item.id)}
          <div class="flex items-end gap-2">
            <div class="min-w-0 flex-1">
              <InputPath
                label=""
                placeholder="ゲームを入れているフォルダを選択"
                path={item.path}
                directory
                withFilter={false}
                browseButtonBorderless
                on:update={(e) => updatePath(i, e.detail.value)}
              />
            </div>
            <button
              type="button"
              class="h-8 w-8 flex shrink-0 items-center justify-center rounded-md bg-transparent text-text-tertiary hover:bg-bg-tertiary hover:text-text-primary"
              on:click={() => removePath(i)}
              aria-label="フォルダを削除"
            >
              <div class="i-iconoir-cancel h-4 w-4" />
            </button>
          </div>
        {/each}
      </div>

      <div class="flex flex-wrap gap-2">
        <Button
          text="フォルダを追加"
          leftIcon="i-iconoir-plus"
          borderless
          on:click={addPath}
        />
        <Button
          text={isLoading ? "判定中" : "紐づけを試す"}
          leftIcon="i-material-symbols:manage-search-rounded"
          variant="accent"
          disabled={!nonEmptyPaths().length || isLoading}
          on:click={runPreview}
        />
      </div>
    </div>

    {#if preview}
      <div class="w-full rounded-md border border-ui-border bg-bg-primary/70 p-4 lg:w-64">
        <div class="text-body2 text-text-tertiary">判定結果</div>
        <div class="mt-1 text-3xl font-bold text-text-primary">{matchRate}%</div>
        <div class="mt-2 text-body2 text-text-secondary">
          {preview.matchedCount} / {preview.scannedFileCount} 件を高信頼で紐づけ
        </div>
      </div>
    {/if}
  </div>

  {#if preview}
    <div class="mt-5 space-y-2">
      {#if preview.results.length === 0}
        <div class="rounded-md border border-ui-border bg-bg-primary/70 p-4 text-body2 text-text-tertiary">
          対象になる exe / lnk / url が見つかりませんでした。
        </div>
      {:else}
        {#each preview.results.slice(0, 12) as result}
          <div class="grid gap-3 rounded-md border border-ui-border bg-bg-primary/70 p-3 md:grid-cols-[minmax(0,1.3fr)_minmax(0,1fr)]">
            <div class="min-w-0">
              <div class="truncate text-body2 font-medium text-text-secondary">
                {filename(result.path)}
              </div>
              <div class="mt-1 truncate text-caption text-text-tertiary">
                {result.path}
              </div>
            </div>
            <div class="min-w-0">
              {#if result.matched}
                <div class="truncate text-body2 font-bold text-text-primary">
                  {result.matched.gamename}
                </div>
                <div class="mt-1 text-caption text-accent-success">高信頼で紐づけ可能</div>
              {:else if result.candidates.length}
                <div class="truncate text-body2 text-text-secondary">
                  候補: {result.candidates[0][1]}
                </div>
                <div class="mt-1 text-caption text-text-tertiary">確認が必要</div>
              {:else}
                <div class="text-body2 text-text-tertiary">候補なし</div>
              {/if}
            </div>
          </div>
        {/each}
        {#if preview.results.length > 12}
          <div class="text-caption text-text-tertiary">
            ほか {preview.results.length - 12} 件
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</section>
