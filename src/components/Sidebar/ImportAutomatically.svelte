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
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import InputPath from "@/components/UI/InputPath.svelte";
  import { get } from "svelte/store";
  import {
    autoImportProgress,
    finishAutoImportProgress,
    incrementAutoImportProgress,
    setAutoImportProgressMessage,
    setAutoImportProgressTotal,
    startAutoImportProgress,
  } from "@/store/autoImportProgress";

  const isDemoBuild = import.meta.env.BASE_URL === "./";
  const demoRegistrationDisabledMessage =
    "demo ではゲーム登録はできません。ホームの「フォルダ紐づけを試す」で判定だけ確認できます。";
  const demoLabels = {
    title: "フォルダから自動追加",
  };

  export let isOpen: boolean;

  let inputContainer: HTMLDivElement | null = null;
  let useCache = true;

  const [paths, getPaths] = createLocalStorageWritable<{ id: number; path: string }[]>(
    "auto-import-dir-paths",
    isDemoBuild
      ? [{ id: 1, path: "C:\\Program Files (x86)\\demo-games" }]
      : [{ id: Math.floor(Math.random() * 100000), path: "" }],
  );

  const updatePath = (index: number, value: string) => {
    if (isDemoBuild) return;
    paths.update((v) => {
      v[index].path = value;
      return v;
    });
  };

  const removePath = (index: number) => {
    if (isDemoBuild) return;
    paths.update((v) => [...v.slice(0, index), ...v.slice(index + 1)]);
  };

  const addEmptyPath = async () => {
    if (isDemoBuild) return;
    if (getPaths().length > 0 && getPaths()[getPaths().length - 1].path === "") {
      return;
    }
    paths.update((v) => {
      v.push({ id: Date.now(), path: "" });
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
    if (get(autoImportProgress).isRunning) {
      showInfoToast("いま別のフォルダ読み取りが進行中です。");
      isOpen = false;
      return;
    }

    startAutoImportProgress();
    setAutoImportProgressMessage("フォルダを読み取り中");
    isOpen = false;
    showInfoToast("フォルダ読み取りを開始しました。ほかの操作を続けられます。");

    const unlistenProgress = await listen<{ message?: string }>("progress", (event) => {
      const message = event.payload?.message?.trim();
      if (message) {
        setAutoImportProgressMessage(message);
      }
    });
    const unlistenProgressLive = await listen<{ max: number | null }>(
      "progresslive",
      (event) => {
        if (typeof event.payload?.max === "number") {
          setAutoImportProgressTotal(event.payload.max);
        } else {
          incrementAutoImportProgress();
        }
      },
    );

    try {
      const res = await commandCreateElementsInPc(
        getPaths().map((v) => v.path),
        useCache,
      );
      await registerCollectionElementDetails();
      await sidebarCollectionElements.refetch();

      const text = res.length
        ? `${res[0]}${res.length === 1 ? "" : ` など${res.length}件`}追加しました`
        : "新しく追加されたゲームはありません";
      showInfoToast(text);
    } finally {
      unlistenProgress();
      unlistenProgressLive();
      finishAutoImportProgress();
    }
  };

  onMount(async () => {
    if (isDemoBuild) {
      return;
    }
    const defaultPaths = await commandGetDefaultImportDirs();
    paths.update((v) => {
      const appendPaths = [];
      for (const defaultPath of defaultPaths) {
        if (!v.some((item) => item.path === defaultPath)) {
          appendPaths.push({
            id: Math.floor(Math.random() * 100000),
            path: defaultPath,
          });
        }
      }
      return [...appendPaths, ...v];
    });
  });
</script>

<Modal
  {isOpen}
  on:close={() => {
    isOpen = false;
  }}
  on:cancel={() => {
    isOpen = false;
  }}
  title={isDemoBuild ? demoLabels.title : "Automatically import game"}
  confirmText="Start import"
  fullmodal
  footerButtonBorderless
  confirmDisabled={!isDemoBuild && (!$paths.length || !$paths.some((v) => v.path) || $autoImportProgress.isRunning)}
  on:confirm={confirm}
>
  <div class="space-y-8">
    {#if !isDemoBuild && $autoImportProgress.isRunning}
      <div class="rounded-md border border-border-primary bg-bg-primary/70 p-4">
        <div class="text-text-primary text-body font-medium">
          フォルダ読み取りはバックグラウンドで進行中です
        </div>
        <div class="mt-1 text-text-tertiary text-body2">
          この画面を閉じても処理は続きます。
        </div>
        <div class="mt-3 text-text-secondary text-body2">
          {$autoImportProgress.message}
          {#if $autoImportProgress.total > 0}
            : {$autoImportProgress.processed}/{$autoImportProgress.total}
          {/if}
        </div>
      </div>
    {/if}

    <div class="space-y-4">
      <div class="text-text-primary text-h4 font-medium">
        自動追加するフォルダ
      </div>
      <form class="flex flex-col gap-2" on:submit|preventDefault={addEmptyPath}>
        {#each $paths as path, i (path.id)}
          <div class="flex items-end gap-8" bind:this={inputContainer}>
            <div class="flex-1">
              <InputPath
                label=""
                placeholder="C:\Program Files (x86)"
                path={path.path}
                directory
                disabled={isDemoBuild || $autoImportProgress.isRunning}
                withFilter={false}
                browseButtonBorderless
                on:update={(e) => updatePath(i, e.detail.value)}
              />
            </div>
            <button
              on:click={() => removePath(i)}
              type="button"
              tabindex={-1}
              disabled={isDemoBuild || $autoImportProgress.isRunning}
              class="ml-auto p-2 bg-transparent disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <div class="w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all" />
            </button>
          </div>
        {/each}
        <Button
          appendClass="m-auto"
          leftIcon="i-iconoir-plus"
          text="Add folder path"
          type="submit"
          disabled={isDemoBuild || $autoImportProgress.isRunning}
          borderless
          on:click={addEmptyPath}
        />
      </form>
    </div>

    <div class="space-y-2">
      <div class="text-text-primary text-h4 font-medium">オプション</div>
      <!-- svelte-ignore a11y-label-has-associated-control -->
      <label
        class="flex gap-2 {(isDemoBuild || $autoImportProgress.isRunning)
          ? 'cursor-not-allowed opacity-50'
          : 'cursor-pointer'}"
      >
        <Checkbox bind:value={useCache} disabled={isDemoBuild || $autoImportProgress.isRunning} />
        <div>
          <div class="text-text-primary text-body font-medium">
            以前にスキャン対象だったファイルのみを対象にする
          </div>
          <div class="text-text-tertiary text-body2">
            自動追加を一度実行した環境ではこのオプションで対象を減らせます。オフの場合、指定したフォルダを最初から再走査します。
          </div>
        </div>
      </label>
    </div>
  </div>
</Modal>
