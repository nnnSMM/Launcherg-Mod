<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Checkbox from "@/components/UI/Checkbox.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import {
    commandCreateElementsInPc,
    commandGetDefaultImportDirs,
  } from "@/lib/command";
  import { showInfoToast } from "@/lib/toast";
  import { createLocalStorageWritable } from "@/lib/utils";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import ModalBase from "@/components/UI/ModalBase.svelte";
  import { fade } from "svelte/transition";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { enqueueGameScreenshotPrefetch } from "@/lib/useGameScreenshots";
  import InputPath from "@/components/UI/InputPath.svelte";

  let isLoading = false;
  const isDemoBuild = import.meta.env.BASE_URL === "./";
  const demoRegistrationDisabledMessage =
    "demo \u3067\u306f\u30b2\u30fc\u30e0\u767b\u9332\u306f\u3067\u304d\u307e\u305b\u3093\u3002\u30db\u30fc\u30e0\u306e\u300c\u30d5\u30a9\u30eb\u30c0\u7d10\u3065\u3051\u3092\u8a66\u3059\u300d\u3067\u5224\u5b9a\u3060\u3051\u78ba\u8a8d\u3067\u304d\u307e\u3059\u3002";
  const demoLabels = {
    title: "\u30d5\u30a9\u30eb\u30c0\u304b\u3089\u81ea\u52d5\u8ffd\u52a0",
    close: "\u9589\u3058\u308b",
    heading: "demo \u3067\u306f\u30b2\u30fc\u30e0\u767b\u9332\u306f\u3067\u304d\u307e\u305b\u3093",
    body: "\u5b9f\u30d5\u30a1\u30a4\u30eb\u306e\u30a2\u30a4\u30b3\u30f3\u53d6\u5f97\u3092\u5b89\u5b9a\u3057\u3066\u518d\u73fe\u3067\u304d\u306a\u3044\u305f\u3081\u3001\u516c\u958b demo \u3067\u306f\u767b\u9332\u51e6\u7406\u3060\u3051\u505c\u6b62\u3057\u3066\u3044\u307e\u3059\u3002\u30db\u30fc\u30e0\u306e\u300c\u30d5\u30a9\u30eb\u30c0\u7d10\u3065\u3051\u3092\u8a66\u3059\u300d\u3067\u5224\u5b9a\u7cbe\u5ea6\u3092\u78ba\u8a8d\u3067\u304d\u307e\u3059\u3002",
  };

  export let isOpen: boolean;

  let inputContainer: HTMLDivElement | null = null;

  let useCache = true;
  const [paths, getPaths] = createLocalStorageWritable<
    { id: number; path: string }[]
  >("auto-import-dir-paths", [
    { id: Math.floor(Math.random() * 100000), path: "" },
  ]);
  const updatePath = (index: number, value: string) => {
    paths.update((v) => {
      v[index].path = value;
      return v;
    });
  };
  const removePath = (index: number) => {
    paths.update((v) => {
      v = [...v.slice(0, index), ...v.slice(index + 1)];
      return v;
    });
  };
  const addEmptyPath = async () => {
    if (
      getPaths().length > 0 &&
      getPaths()[getPaths().length - 1].path === ""
    ) {
      return;
    }
    paths.update((v) => {
      v.push({ id: new Date().getTime(), path: "" });
      return v;
    });
    await new Promise((resolve) => setTimeout(resolve, 0));
    if (inputContainer) {
      const inputs = inputContainer.getElementsByTagName("input");
      if (inputs.length > 0) {
        inputs[inputs.length - 1].focus();
      }
    }
  };
  const confirm = async () => {
    if (isDemoBuild) {
      showInfoToast(demoRegistrationDisabledMessage);
      isOpen = false;
      return;
    }
    isLoading = true;
    const beforeIds = new Set(sidebarCollectionElements.value().map((v) => v.id));
    const res = await commandCreateElementsInPc(
      getPaths().map((v) => v.path),
      useCache,
    );
    await registerCollectionElementDetails();
    await sidebarCollectionElements.refetch();
    enqueueGameScreenshotPrefetch(
      sidebarCollectionElements.value().filter((v) => !beforeIds.has(v.id)),
    );

    isLoading = false;

    const text = res.length
      ? `「${res[0]}」${
          res.length === 1 ? "が" : `、他${res.length}件`
        }追加されました`
      : "新しく追加されたゲームはありません";

    showInfoToast(text);
    isOpen = false;
  };

  let processFileNums = 0;
  let processedFileNums = 0;

  onMount(async () => {
    const defaultPaths = await commandGetDefaultImportDirs();
    paths.update((v) => {
      const appendPaths = [];
      for (const defaultPath of defaultPaths) {
        if (!v.some((v) => v.path === defaultPath)) {
          appendPaths.push({
            id: Math.floor(Math.random() * 100000),
            path: defaultPath,
          });
        }
      }
      return [...appendPaths, ...v];
    });
    // const unlistenProgress = await listen<{ message: string }>(
    //   "progress",
    //   (event) => {
    //     showInfoToast(event.payload.message, 10000);
    //   }
    // );
    const unlistenProgressLive = await listen<{ max: number | null }>(
      "progresslive",
      (event) => {
        if (event.payload.max) {
          processFileNums = event.payload.max;
        } else {
          processedFileNums = processedFileNums + 1;
        }
      },
    );
    return () => {
      // unlistenProgress();
      unlistenProgressLive();
    };
  });
</script>

{#if !isLoading}
  <Modal
    {isOpen}
    on:close={() => {
      if (!isLoading) {
        isOpen = false;
      }
    }}
    on:cancel={() => {
      if (!isLoading) {
        isOpen = false;
      }
    }}
    title={isDemoBuild ? demoLabels.title : "Automatically import game"}
    confirmText={isDemoBuild ? demoLabels.close : "Start import"}
    fullmodal
    footerButtonBorderless
    confirmDisabled={!isDemoBuild && (!$paths.length || !$paths.some((v) => v.path) || isLoading)}
    on:confirm={confirm}
  >
    <div class="space-y-8">
      {#if isDemoBuild}
        <div class="space-y-2">
          <div class="text-text-primary text-h4 font-medium">{demoLabels.heading}</div>
          <div class="text-text-tertiary text-body2">{demoLabels.body}</div>
        </div>
      {:else}
      <div class="space-y-4">
        <div class="text-text-primary text-h4 font-medium">
          自動追加するフォルダ
        </div>
        <form
          class="flex flex-col gap-2"
          on:submit|preventDefault={addEmptyPath}
        >
          {#each $paths as path, i (path.id)}
            <div class="flex items-end gap-8" bind:this={inputContainer}>
              <div class="flex-1">
                <InputPath
                  label=""
                  placeholder="C:\Program Files (x86)"
                  path={path.path}
                  directory
                  withFilter={false}
                  browseButtonBorderless
                  on:update={(e) => updatePath(i, e.detail.value)}
                />
              </div>
              <button
                on:click={() => removePath(i)}
                type="button"
                tabindex={-1}
                class="ml-auto p-2 bg-transparent"
              >
                <div
                  class="w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all"
                />
              </button>
            </div>
          {/each}
          <Button
            appendClass="m-auto"
            leftIcon="i-iconoir-plus"
            text="Add folder path"
            type="submit"
            borderless
            on:click={addEmptyPath}
          />
        </form>
      </div>
      <div class="space-y-2">
        <div class="text-text-primary text-h4 font-medium">オプション</div>
        <!-- svelte-ignore a11y-label-has-associated-control -->
        <label class="flex gap-2 cursor-pointer">
          <Checkbox bind:value={useCache} />
          <div>
            <div class="text-text-primary text-body font-medium">
              前回のスキャン以降に追加されたファイルのみを対象にする
            </div>
            <div class="text-text-tertiary text-body2">
              自動追加が初回の場合このオプションは意味を持ちません。このオプションがオフの場合、自動追加は2分程度かかる場合があります。
            </div>
          </div>
        </label>
      </div>
      {/if}
    </div>
  </Modal>
{:else if isLoading}
  <div transition:fade={{ delay: 150 }}>
    <ModalBase isOpen={true} panelClass="max-w-82">
      <div class="flex flex-col items-center justify-center gap-5 w-full p-12">
        <div
          class="w-20 h-20 border-12px border-solid border-#D9D9D9 border-t-#2D2D2D border-t-rounded rounded-full animate-spin"
        />
        <div class="text-text-primary text-h3 font-bold">処理中</div>
        {#if processFileNums}
          <div class="text-text-primary text-body font-medium">
            処理したファイル: {processedFileNums}/{processFileNums}
          </div>
        {/if}
      </div>
    </ModalBase>
  </div>
{/if}
