<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "/icon.png";

  const repo = "https://github.com/nnnSMM/Launcherg-Mod";
  const usage = `${repo}/blob/main/USAGE.md`;
  const releaseApi = "https://api.github.com/repos/nnnSMM/Launcherg-Mod/releases/latest";
  const fallbackDownloadZip =
    `${repo}/releases/download/20260505/Launcherg-Mod_20260505_x64_ja-JP.zip`;
  const rawImages = "images";
  const autoSwitchMs = 2000;
  const pageTitle =
    "Launcherg-Mod | VN・ノベルゲームのプレイ時間とスクリーンショット管理";
  const pageDescription =
    "Launcherg-Mod は、PC上のノベルゲームを登録して起動し、プレイ時間、スクリーンショット、メモをゲームごとに整理できる Windows アプリです。";

  const features = [
    {
      icon: "i-material-symbols:grid-view-rounded",
      title: "ノベルゲームを一元管理",
      body: "実行ファイルやショートカットを登録\nライブラリからすぐに起動",
    },
    {
      icon: "i-material-symbols:timer-outline-rounded",
      title: "プレイ時間を記録",
      body: "起動したゲームのプレイ時間を\nタイトルごとに記録",
    },
    {
      icon: "i-material-symbols:query-stats-rounded",
      title: "プレイ傾向を振り返る",
      body: "プレイ時間や日別の記録を\n統計画面で確認",
    },
    {
      icon: "i-material-symbols:photo-camera-outline-rounded",
      title: "スクリーンショットを整理",
      body: "撮影したスクリーンショットを\nプレイ中のゲームに関連付け",
    },
    {
      icon: "i-material-symbols:edit-note-rounded",
      title: "メモを残す",
      body: "攻略メモや感想をMarkdownで保存",
    },
  ];

  const screenshotGroups = [
    {
      title: "ライブラリ",
      caption: "登録したノベルゲームを一覧で確認できます。",
      images: [
        {
          title: "ライブラリ画面",
          src: `${rawImages}/main_window_1.png`,
        },
      ],
    },
    {
      title: "ゲーム詳細",
      caption: "起動、メモ、プレイ履歴をゲームごとに管理できます。",
      images: [
        {
          title: "ゲーム詳細画面",
          src: `${rawImages}/game_details.png`,
        },
      ],
    },
    {
      title: "統計",
      caption: "プレイ時間、プレイ状況、日別のアクティビティをまとめて確認できます。",
      images: [
        {
          title: "統計画面",
          src: `${rawImages}/stats.png`,
        },
      ],
    },
    {
      title: "スクリーンショット",
      caption: "スクリーンショット一覧と閲覧画面を切り替えて確認できます。",
      images: [
        {
          title: "一覧",
          src: `${rawImages}/screenshots_tab.png`,
        },
        {
          title: "閲覧",
          src: `${rawImages}/screenshot_view.png`,
        },
      ],
    },
  ];

  const faqItems = [
    [
      "対象ゲームは？",
      "エロゲー批評空間に登録されているノベルゲームを対象にしています。",
    ],
    [
      "データ収集はありますか？",
      "ユーザーのデータは収集しません。",
    ],
    [
      "詳細な使い方は？",
      "左の「使い方を見る」から確認できます。",
    ],
    [
      "Launchergとの関係は？",
      "ryoha000さんのLaunchergをクローンし、独自に改造した派生版です。",
    ],
  ];

  let groupIndex = 0;
  let imageIndex = 0;
  let intervalId: number | null = null;
  let downloadZip = fallbackDownloadZip;

  $: activeGroup = screenshotGroups[groupIndex] ?? screenshotGroups[0];
  $: activeImages = activeGroup.images;
  $: activeImage = activeImages[imageIndex % activeImages.length] ?? activeImages[0];

  const resetAutoSwitch = (imageCount = activeImages.length) => {
    imageIndex = 0;
    if (intervalId) {
      window.clearInterval(intervalId);
      intervalId = null;
    }
    if (imageCount > 1) {
      intervalId = window.setInterval(() => {
        imageIndex = (imageIndex + 1) % imageCount;
      }, autoSwitchMs);
    }
  };

  const selectGroup = (index: number) => {
    groupIndex = index;
    resetAutoSwitch(screenshotGroups[index]?.images.length ?? 1);
  };

  const updateLatestDownloadZip = async () => {
    try {
      const response = await fetch(releaseApi, {
        headers: { Accept: "application/vnd.github+json" },
      });
      if (!response.ok) {
        return;
      }
      const release = (await response.json()) as {
        assets?: Array<{ name?: string; browser_download_url?: string }>;
      };
      const zipAsset = release.assets?.find(
        (asset) => asset.name?.endsWith(".zip") && asset.browser_download_url,
      );
      if (zipAsset?.browser_download_url) {
        downloadZip = zipAsset.browser_download_url;
      }
    } catch {
      // Keep the bundled fallback URL when GitHub API is unavailable.
    }
  };

  onMount(() => {
    resetAutoSwitch();
    void updateLatestDownloadZip();
    return () => {
      if (intervalId) {
        window.clearInterval(intervalId);
      }
    };
  });
</script>

<svelte:head>
  <title>{pageTitle}</title>
  <meta
    name="description"
    content={pageDescription}
  />
</svelte:head>

<div class="landing h-full overflow-y-auto bg-[#11100e] text-[#f6f0e7]">
  <header class="sticky top-0 z-30 border-b border-[#3a332d] bg-[#11100e]/95">
    <div class="mx-auto flex max-w-6xl items-center justify-between gap-3 px-4 py-3 sm:px-6">
      <a href="#/" class="inline-flex min-w-0 items-center gap-2 text-sm font-bold">
        <img
          src={Icon}
          alt="Launcherg-Mod icon"
          class="h-6 w-6 rounded-[4px] object-contain"
        />
        <span>Launcherg-Mod</span>
      </a>
      <nav class="flex shrink-0 items-center gap-2 text-sm">
        <a
          href="#/demo"
          class="hidden rounded-md px-3 py-2 text-[#cfc5b8] transition hover:bg-[#24211e] hover:text-[#f6f0e7] sm:inline-flex"
        >
          デモを試す
        </a>
        <a
          href={usage}
          target="_blank"
          rel="noreferrer"
          class="hidden rounded-md px-3 py-2 text-[#cfc5b8] transition hover:bg-[#24211e] hover:text-[#f6f0e7] sm:inline-flex"
        >
          Guide
        </a>
        <a
          href={repo}
          target="_blank"
          rel="noreferrer"
          class="inline-flex min-h-9 items-center gap-2 rounded-md border border-[#5b5048] px-3 py-2 font-semibold text-[#f6f0e7] transition hover:bg-[#24211e]"
        >
          <span class="i-material-symbols:code-rounded h-4 w-4" />
          GitHub
        </a>
      </nav>
    </div>
  </header>

  <section
    class="relative overflow-hidden border-b border-[#3a332d] bg-[#151310]"
    style={`background-image: url("${rawImages}/main_window_1.png"); background-size: cover; background-position: center top;`}
  >
    <div class="absolute inset-0 bg-[#11100e]/72" />
    <div class="absolute inset-x-0 bottom-0 h-28 bg-[#11100e]/88" />
    <div class="relative mx-auto flex min-h-[68svh] max-w-6xl items-end px-4 py-12 sm:px-6 sm:py-16">
      <div class="max-w-3xl pb-2">
        <div class="mb-4 inline-flex items-center rounded-md border border-[#3bb7a9]/40 bg-[#143530]/90 px-3 py-1 text-xs font-semibold text-[#a8eee7]">
          VN・ノベルゲーム管理アプリ
        </div>
        <h1 class="break-words text-5xl font-bold leading-[1.05] text-[#fffaf2] sm:text-6xl">
          Launcherg-Mod
        </h1>
        <p class="mt-5 max-w-2xl text-base leading-8 text-[#e4d8c9] sm:text-lg">
          散らばりがちなノベルゲームをひとつのライブラリにまとめ、ショートカット起動、プレイ時間の記録、スクリーンショット整理、メモ管理を同じ場所で扱う Windows アプリです。
        </p>
        <div class="mt-7 flex flex-col gap-3 sm:flex-row">
          <a
            href={downloadZip}
            target="_blank"
            rel="noreferrer"
            download
            class="inline-flex min-h-11 items-center justify-center gap-2 rounded-md bg-[#df6d5b] px-4 py-2 text-sm font-bold text-[#170f0d] transition hover:bg-[#ef8270]"
          >
            <span class="i-material-symbols:download-rounded h-5 w-5" />
            最新版をダウンロード
          </a>
          <a
            href="#/demo"
            class="inline-flex min-h-11 items-center justify-center gap-2 rounded-md border border-[#5b5048] bg-[#161411]/90 px-4 py-2 text-sm font-bold text-[#f6f0e7] transition hover:bg-[#24211e]"
          >
            デモを試す
            <span class="i-material-symbols:chevron-right-rounded h-5 w-5" />
          </a>
        </div>
      </div>
    </div>
  </section>

  <section class="border-b border-[#3a332d] bg-[#11100e]">
    <div class="mx-auto max-w-6xl px-4 py-12 sm:px-6 sm:py-14">
      <div class="mb-7 flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
        <div class="max-w-2xl">
          <p class="mb-2 text-xs font-bold uppercase tracking-[0.22em] text-[#3bb7a9]">
            Features
          </p>
          <h2 class="text-2xl font-bold tracking-tight text-[#fffaf2] sm:text-3xl">
            できること
          </h2>
          <p class="mt-2 text-sm leading-6 text-[#b6aa9c]">
            ゲームの登録、起動、記録、整理を、日常的に使いやすい形でまとめます。
          </p>
        </div>
      </div>

      <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
        {#each features as feature}
          <article class="group relative overflow-hidden rounded-xl border border-[#3a332d] bg-[#191612] p-4 shadow-[0_16px_50px_rgba(0,0,0,0.18)] transition hover:-translate-y-0.5 hover:border-[#5b5048] hover:bg-[#211d18]">
            <div class="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-[#3bb7a9]/55 to-transparent opacity-0 transition group-hover:opacity-100" />
            <div class="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-[#143530] text-[#a8eee7] ring-1 ring-[#3bb7a9]/20">
              <span class={`${feature.icon} h-5 w-5`} />
            </div>
            <h3 class="font-bold text-[#fffaf2]">{feature.title}</h3>
            <p class="mt-1.5 whitespace-pre-line text-sm leading-6 text-[#b6aa9c]">{feature.body}</p>
          </article>
        {/each}
      </div>
    </div>
  </section>

  <section class="border-b border-[#3a332d] bg-[#171512]">
    <div class="mx-auto max-w-6xl px-4 py-12 sm:px-6 sm:py-14">
      <div class="mb-6 flex flex-col justify-between gap-4 md:flex-row md:items-end">
        <div class="max-w-2xl">
          <p class="mb-2 text-xs font-bold uppercase tracking-[0.22em] text-[#3bb7a9]">
            Screens
          </p>
          <h2 class="text-2xl font-bold tracking-tight text-[#fffaf2] sm:text-3xl">
            主要画面
          </h2>
          <p class="mt-2 text-sm leading-6 text-[#b6aa9c]">
            ライブラリ、ゲーム詳細、スクリーンショット管理を通じて、登録したゲームの状態を確認できます。
          </p>
        </div>

        {#if activeImages.length > 1}
          <div
            class="flex items-center gap-3 rounded-full border border-[#3a332d] bg-[#11100e]/80 px-3 py-2 text-xs text-[#b6aa9c] shadow-lg shadow-black/10"
            aria-live="polite"
          >
            <span>{autoSwitchMs / 1000}秒ごとに切替</span>
            <div class="flex gap-1.5" aria-label={`${activeGroup.title} の表示中画像`}>
              {#each activeImages as image, index}
                <span
                  class={`h-2 w-2 rounded-full transition ${index === imageIndex % activeImages.length ? "bg-[#3bb7a9] shadow-[0_0_10px_rgba(59,183,169,0.65)]" : "bg-[#5b5048]"}`}
                  title={image.title}
                />
              {/each}
            </div>
            <span class="text-[#8f8377]">{activeImage.title}</span>
          </div>
        {/if}
      </div>

      <div class="grid gap-5 lg:grid-cols-[260px_1fr]">
        <div class="flex gap-2 overflow-x-auto pb-1 lg:flex-col lg:overflow-visible lg:pb-0">
          {#each screenshotGroups as group, index}
            <button
              type="button"
              on:click={() => selectGroup(index)}
              class={`shrink-0 rounded-xl px-4 py-3 text-left text-sm font-semibold transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#3bb7a9] lg:w-full ${
                groupIndex === index
                  ? "bg-[#3bb7a9] text-[#08211e] shadow-lg shadow-[#3bb7a9]/15"
                  : "border border-[#3a332d] bg-[#11100e] text-[#cfc5b8] hover:border-[#5b5048] hover:bg-[#24211e] hover:text-[#f6f0e7]"
              }`}
            >
              <span class="block">{group.title}</span>
              <span
                class={`mt-1 block text-xs font-normal leading-5 ${
                  groupIndex === index ? "text-[#0c3430]" : "text-[#8f8377]"
                }`}
              >
                {group.caption}
              </span>
            </button>
          {/each}
        </div>

        <figure class="overflow-hidden rounded-2xl border border-[#3a332d] bg-[#11100e] p-3 shadow-2xl shadow-black/25">
          <div class="overflow-hidden rounded-xl bg-[#0b0a09] ring-1 ring-white/[0.04]">
            <img
              src={activeImage.src}
              alt={`${activeGroup.title} - ${activeImage.title}`}
              loading="lazy"
              class="screenshot-image block h-auto w-full object-contain"
            />
          </div>
          <figcaption class="px-1.5 pt-3 text-sm leading-6 text-[#b6aa9c]">
            {activeGroup.caption}
          </figcaption>
        </figure>
      </div>
    </div>
  </section>

  <section class="bg-[#11100e]">
    <div class="mx-auto grid max-w-6xl gap-4 px-4 py-12 sm:px-6 sm:py-14 lg:grid-cols-[0.95fr_1.05fr]">
      <div class="relative overflow-hidden rounded-2xl border border-[#8d6b2e]/85 bg-gradient-to-br from-[#211b12] to-[#17120c] p-5 shadow-xl shadow-black/15 sm:p-6">
        <div class="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-[#d8ad50]/70 to-transparent" />
        <h2 class="text-2xl font-bold text-[#fffaf2]">最新版をダウンロード</h2>
        <p class="mt-3 text-sm leading-7 text-[#d5c7b5]">
          zip ファイルを直接ダウンロードし、展開後にインストーラーを実行してください。
        </p>
        <div class="mt-5 flex flex-col gap-3 sm:flex-row">
          <a
            href={downloadZip}
            target="_blank"
            rel="noreferrer"
            download
            class="inline-flex min-h-10 items-center justify-center gap-2 rounded-xl bg-[#d8ad50] px-4 py-2 text-sm font-bold text-[#171108] shadow-lg shadow-[#d8ad50]/15 transition hover:bg-[#edc164] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#d8ad50] focus-visible:ring-offset-2 focus-visible:ring-offset-[#11100e]"
          >
            <span class="i-material-symbols:download-rounded h-5 w-5" />
            ダウンロード
          </a>
          <a
            href={usage}
            target="_blank"
            rel="noreferrer"
            class="inline-flex min-h-10 items-center justify-center gap-2 rounded-xl border border-[#5b5048] px-4 py-2 text-sm font-bold text-[#f6f0e7] transition hover:border-[#77695e] hover:bg-[#24211e] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#3bb7a9] focus-visible:ring-offset-2 focus-visible:ring-offset-[#11100e]"
          >
            使い方を見る
            <span class="i-material-symbols:open-in-new-rounded h-4 w-4" />
          </a>
        </div>
      </div>

      <div class="rounded-2xl border border-[#3a332d] bg-[#1a1714] p-5 shadow-xl shadow-black/10 sm:p-6">
        <h2 class="text-2xl font-bold text-[#fffaf2]">FAQ</h2>
        <div class="mt-4 divide-y divide-[#3a332d]">
          {#each faqItems as [question, answer]}
            <details class="group py-3">
              <summary class="flex cursor-pointer list-none items-center justify-between gap-3 text-sm font-semibold text-[#f6f0e7]">
                {question}
                <span class="i-material-symbols:chevron-right-rounded h-5 w-5 shrink-0 text-[#8f8377] transition group-open:rotate-90 group-open:text-[#3bb7a9]" />
              </summary>
              <p class="mt-2 text-sm leading-6 text-[#b6aa9c]">{answer}</p>
            </details>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <footer class="border-t border-[#3a332d] bg-[#0d0c0b] px-4 py-6 sm:px-6">
    <div class="mx-auto flex max-w-6xl flex-col gap-3 text-sm text-[#8f8377] sm:flex-row sm:items-center sm:justify-between">
      <span>Launcherg-Mod</span>
      <div class="flex flex-wrap gap-4">
        <a class="hover:text-[#f6f0e7]" href={repo} target="_blank" rel="noreferrer">GitHub</a>
        <a class="hover:text-[#f6f0e7]" href={usage} target="_blank" rel="noreferrer">Usage</a>
        <a class="hover:text-[#f6f0e7]" href={downloadZip} target="_blank" rel="noreferrer" download>Download</a>
      </div>
    </div>
  </footer>
</div>

<style>
  .screenshot-image {
    animation: screenshot-fade 240ms ease-out;
  }

  @keyframes screenshot-fade {
    from {
      opacity: 0.72;
      transform: translateY(2px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .screenshot-image {
      animation: none;
    }

    * {
      scroll-behavior: auto !important;
    }
  }
</style>
