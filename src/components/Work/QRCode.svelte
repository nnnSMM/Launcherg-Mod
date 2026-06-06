<script lang="ts">
  import Modal from "@/components/UI/Modal.svelte";
  import QrCodeCanvas from "@/components/UI/QRCodeCanvas.svelte";
  import { showInfoToast } from "@/lib/toast";

  export let isOpen: boolean;
  export let id: number | undefined = undefined;
  export let seiyaUrl: string = "";

  let readyPromise: Promise<string> | undefined = undefined;
  let readyRequestKey: string | undefined = undefined;

  const connectSkyWay = async (workId: number | undefined, url: string) => {
    const { skyWay } = await import("@/store/skyway");
    return skyWay.connect(workId, url);
  };

  $: {
    if (isOpen) {
      const nextRequestKey = `${id ?? "library"}:${seiyaUrl}`;
      if (readyRequestKey !== nextRequestKey) {
        readyRequestKey = nextRequestKey;
        readyPromise = connectSkyWay(id, seiyaUrl);
      }
    } else {
      readyRequestKey = undefined;
      readyPromise = undefined;
    }
  }

  const copyUrlToClipboard = async (value: string) => {
    await navigator.clipboard.writeText(value);
    showInfoToast("クリップボードにコピーしました");
  };

  const getDisplayUrl = (value: string) => {
    const url = new URL(value);
    return `${url.origin}${url.pathname}${url.search}${url.hash}`;
  };
</script>

<Modal
  {isOpen}
  on:close={() => (isOpen = false)}
  on:cancel={() => (isOpen = false)}
  title="補助コントローラー連携"
  autofocusCloseButton
  withFooter={false}
>
  <div class="space-y-4 text-text-primary">
    <div>
      このQRから補助コントローラーを開きます。PCで起動中のゲームを自動検知して連携します。
    </div>
    {#if readyPromise}
      {#await readyPromise}
        <div class="flex flex-col items-center justify-center gap-5 w-full p-12">
          <div
            class="w-20 h-20 border-12px border-solid border-#D9D9D9 border-t-#2D2D2D border-t-rounded rounded-full animate-spin"
          />
          <div class="text-text-primary text-h3 font-bold">準備中</div>
        </div>
      {:then value}
        <div class="flex flex-col justify-center items-center gap-4">
          <button
            on:click={() => copyUrlToClipboard(value)}
            class="flex hover:bg-bg-button rounded px-4 py-1 items-center gap-4 bg-inherit"
          >
            <div
              class="i-material-symbols-content-copy-outline-rounded w-5 h-5"
            />
            <div class="max-w-[360px] break-all text-left text-xs">
              {getDisplayUrl(value)}
            </div>
          </button>
          <QrCodeCanvas {value} />
        </div>
      {/await}
    {/if}
  </div>
</Modal>
