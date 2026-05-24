<script lang="ts">
    import { link } from "svelte-spa-router";
    import { PlayStatus, type CollectionElement, type Work } from "@/lib/types";
    import { playStatusLabel } from "@/lib/playStatus";
    import { seiya } from "@/store/seiya";
    import LinkToSidebar from "@/components/Work/LinkToSidebar.svelte";
    import Detail from "@/components/Work/Detail.svelte";
    import ScreenshotGallery from "@/components/Work/ScreenshotGallery.svelte";
    import { formatLastPlayed, formatPlayTime } from "@/lib/utils";
    import { memo } from "@/store/memo";
    import { onMount, onDestroy } from "svelte";
    import { readImage } from "@tauri-apps/plugin-clipboard-manager";
    import { commandUploadImage, commandSaveScreenshotByPid } from "@/lib/command";
    import { startProcessMap } from "@/store/startProcessMap";
    import { showErrorToast } from "@/lib/toast";
    import EasyMDE from "easymde";
    import { open } from "@tauri-apps/plugin-dialog";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { skyWay } from "@/store/skyway";

    export let work: Work;
    export let element: CollectionElement;
    export let page: "overview" | "record" | "memo" | "screenshots" =
        "overview";

    const mde = (node: HTMLElement) => {
        const easyMDE = new EasyMDE({
            element: node,
            spellChecker: false,
            sideBySideFullscreen: false,
            previewImagesInEditor: true,
            autofocus: false,
            autosave: {
                enabled: true,
                delay: 1000,
                uniqueId: `memo-${work.id}`,
            },
            toolbar: [
                "bold",
                "italic",
                "heading",
                "|",
                "quote",
                "unordered-list",
                "ordered-list",
                "|",
                "link",
                {
                    name: "image",
                    action: async () => {
                        const selected = await open({
                            multiple: false,
                            filters: [
                                {
                                    name: "Image",
                                    extensions: ["png", "jpeg", "jpg", "*"],
                                },
                            ],
                        });
                        if (selected === null || Array.isArray(selected)) {
                            return;
                        }
                        insertImage(selected.path);
                    },
                    className: "fa fa-picture-o",
                    title: "Insert image",
                },
                {
                    name: "screenshot",
                    action: async () => {
                        const startProcessId = $startProcessMap[work.id];
                        try {
                            const screenshotPath = await commandSaveScreenshotByPid(
                                work.id,
                                startProcessId,
                            );
                            insertImage(screenshotPath);
                        } catch (e) {
                            showErrorToast("スクリーンショットの取得に失敗しました");
                            console.error(e);
                        }
                    },
                    className: "fa fa-desktop",
                    title: "Insert screenshot",
                },
            ],
            imagesPreviewHandler: (imagePath) => convertFileSrc(imagePath),
        });

        let destroyed = false;

        const onPaste = async () => {
            try {
                if (destroyed) return;
                const image = await readImage();
                if (destroyed) return;
                const rgba = await image.rgba();
                const size = await image.size();
                if (destroyed) return;

                const canvas = document.createElement("canvas");
                canvas.width = size.width;
                canvas.height = size.height;
                const ctx = canvas.getContext("2d");
                if (!ctx) return;
                const imageData = new ImageData(
                    new Uint8ClampedArray(rgba),
                    size.width,
                    size.height,
                );
                ctx.putImageData(imageData, 0, 0);
                const base64Image = canvas.toDataURL("image/png").split(",")[1];

                const imagePath = await commandUploadImage(work.id, base64Image);
                if (destroyed) return;
                insertImage(imagePath);
            } catch {}
        };

        const insertImage = (imagePath: string) => {
            if (destroyed) return;
            const cursor = easyMDE.codemirror.getCursor();
            const prev = easyMDE.value();
            const lines = prev.split("\n");
            const newLines: string[] = [];
            for (let i = 0; i < lines.length; i++) {
                newLines.push(lines[i]);
                if (i === cursor.line) {
                    newLines.push(`![](${imagePath})`);
                    newLines.push("");
                }
            }
            easyMDE.codemirror.setValue(newLines.join("\n"));
            easyMDE.codemirror.setCursor({ line: cursor.line + 2, ch: 0 });
        };

        const ele = node.closest(".EasyMDEContainer") || node.parentElement;
        const styleTimer = window.setTimeout(() => {
            if (destroyed) return;
            const container = node.parentElement?.querySelector(".EasyMDEContainer") || node.parentElement;
            if (container) {
                const toolbar = container.querySelector<HTMLElement>(".editor-toolbar");
                if (toolbar) {
                    toolbar.style.backgroundColor = "rgba(45, 51, 59, 0.2)";
                    toolbar.style.backdropFilter = "blur(12px)";
                    toolbar.style.border = "1px solid rgba(255, 255, 255, 0.08)";
                    toolbar.style.borderBottom = "none";
                    toolbar.style.borderTopLeftRadius = "8px";
                    toolbar.style.borderTopRightRadius = "8px";

                    const buttons = toolbar.querySelectorAll<HTMLElement>("button");
                    buttons.forEach((btn) => {
                        btn.style.backgroundColor = "transparent";
                        btn.style.color = "var(--color-text-primary, #ffffff)";
                        btn.style.border = "none";
                        btn.style.borderRadius = "4px";
                        btn.style.transition = "all 0.2s ease";
                        
                        btn.addEventListener("mouseenter", () => {
                            btn.style.backgroundColor = "rgba(255, 255, 255, 0.1)";
                        });
                        btn.addEventListener("mouseleave", () => {
                            btn.style.backgroundColor = "transparent";
                        });
                    });

                    const separators = toolbar.querySelectorAll<HTMLElement>(".separator");
                    separators.forEach((sep) => {
                        sep.style.borderLeft = "1px solid rgba(255, 255, 255, 0.1)";
                        sep.style.borderRight = "none";
                    });
                }
                const codeMirror = container.querySelector<HTMLElement>(".CodeMirror");
                if (codeMirror) {
                    codeMirror.style.backgroundColor = "rgba(34, 39, 46, 0.2)";
                    codeMirror.style.backdropFilter = "blur(12px)";
                    codeMirror.style.border = "1px solid rgba(255, 255, 255, 0.08)";
                    codeMirror.style.borderBottomLeftRadius = "8px";
                    codeMirror.style.borderBottomRightRadius = "8px";
                }
            }
        }, 0);

        ele?.addEventListener("paste", onPaste);

        const syncTimer = setInterval(() => {
            if (destroyed) return;
            const current = easyMDE.value();
            if ($memo.find((v) => v.workId === work.id)?.value !== current) {
                memo.update((memos) =>
                    memos.reduce(
                        (acc, cur) => {
                            if (cur.workId !== work.id) acc.push(cur);
                            return acc;
                        },
                        [
                            { workId: work.id, value: current, lastModified: "local" },
                        ] as typeof $memo,
                    ),
                );
                skyWay.syncMemo(work.id, current);
            }
        }, 1000);

        const unsubscribe = memo.subscribe((memos) => {
            if (destroyed) return;
            const targetMemo = memos.find((v) => v.workId === work.id);
            if (targetMemo?.lastModified === "remote" && easyMDE.value() !== targetMemo.value) {
                easyMDE.value(targetMemo.value);
            }
        });

        return {
            destroy: () => {
                destroyed = true;
                ele?.removeEventListener("paste", onPaste);
                unsubscribe();
                clearInterval(syncTimer);
                clearTimeout(styleTimer);
                easyMDE.cleanup();
                const wrapper = easyMDE.codemirror.getWrapperElement();
                const container = wrapper.parentElement;
                if (node.isConnected && container?.parentElement) {
                    easyMDE.toTextArea();
                }
            },
        };
    };

    $: seiyaUrlPromise = work ? seiya.getUrl(work.name) : Promise.resolve("");

    const formatDate = (value: string | null | undefined) => {
        if (!value) return "未記録";
        return new Date(value).toLocaleDateString("ja-JP");
    };

    const normalizeDescription = (value: string | null | undefined) =>
        value
            ?.replace(/[ \t]+/g, " ")
            .replace(/\n{3,}/g, "\n\n")
            .trim() ?? "";

    $: recordRows = element
        ? [
        {
            label: "総プレイ時間",
            value: formatPlayTime(element.totalPlayTimeSeconds),
            icon: "i-material-symbols-hourglass-outline-rounded",
        },
        {
            label: "初プレイ",
            value: formatDate(element.firstPlayAt),
            icon: "i-material-symbols-play-circle-outline-rounded",
        },
        {
            label: "最終プレイ",
            value: formatLastPlayed(element.lastPlayAt) || formatDate(element.lastPlayAt),
            icon: "i-material-symbols-history-rounded",
        },
        {
            label: "プレイ状況",
            value: playStatusLabel[element.playStatus] ?? playStatusLabel[PlayStatus.Unplayed],
            icon: "i-material-symbols-check-circle-outline-rounded",
        },
    ]
        : [];

    $: overviewBrand = work?.brandName || element?.brandname || "ブランド未登録";
    $: overviewDescription = normalizeDescription(work?.description);
</script>

{#if work && work.id && element}
<div>
    {#if page === "overview"}
        <div class="grid grid-cols-1 2xl:grid-cols-[minmax(0,1fr)_minmax(340px,0.52fr)] gap-5 lg:gap-6">
            <div class="min-w-0">
                <section
                    id="work-overview"
                    class="rounded-lg border border-border-primary bg-bg-primary/38 backdrop-blur-md shadow-sm overflow-hidden"
                    aria-labelledby="work-overview-title"
                >
                    <div class="p-4 lg:p-5 border-b border-border-primary">
                        <div class="flex flex-col gap-3 lg:flex-row lg:items-start">
                            <div class="min-w-0 flex-1">
                                <h2 id="work-overview-title" class="text-h3 text-text-primary font-bold">
                                    概要
                                </h2>
                                <p class="text-body3 text-text-tertiary mt-1 truncate">
                                    {overviewBrand}
                                </p>
                                {#if overviewDescription}
                                    <p class="mt-4 max-w-4xl text-body2 leading-7 text-text-secondary whitespace-pre-line break-words">
                                        {overviewDescription}
                                    </p>
                                {:else}
                                    <p class="mt-4 text-body3 text-text-tertiary">
                                        説明文は未取得です。
                                    </p>
                                {/if}
                            </div>
                            <div class="flex flex-wrap gap-2">
                                <a
                                    href={work.officialHomePage}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="inline-flex items-center gap-1 rounded border border-border-primary px-3 py-1.5 text-body3 text-text-link hover:bg-bg-button-hover hover:text-text-primary focus-visible:ring-2 focus-visible:ring-accent-accent"
                                >
                                    Official
                                    <div class="i-material-symbols-open-in-new text-sm" />
                                </a>
                                <a
                                    href={`https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=${work.id}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="inline-flex items-center gap-1 rounded border border-border-primary px-3 py-1.5 text-body3 text-text-link hover:bg-bg-button-hover hover:text-text-primary focus-visible:ring-2 focus-visible:ring-accent-accent"
                                >
                                    ErogameScape
                                    <div class="i-material-symbols-open-in-new text-sm" />
                                </a>
                                {#await seiyaUrlPromise then url}
                                    <a
                                        href={url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-center gap-1 rounded border border-border-primary px-3 py-1.5 text-body3 text-text-link hover:bg-bg-button-hover hover:text-text-primary focus-visible:ring-2 focus-visible:ring-accent-accent"
                                    >
                                        誠也の部屋
                                        <div class="i-material-symbols-open-in-new text-sm" />
                                    </a>
                                {/await}
                            </div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3">
                        <div class="p-4 border-b md:border-r border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">ブランド</div>
                            <div class="text-body2 text-text-link font-medium truncate">
                                <LinkToSidebar value={work.brandName} />
                            </div>
                        </div>
                        <div class="p-4 border-b xl:border-r border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">発売日</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {work.sellday || element.sellday || "未登録"}
                            </div>
                        </div>
                        <div class="p-4 border-b md:border-r xl:border-r-0 border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">平均プレイ時間</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {work.statistics.playTime}
                            </div>
                        </div>
                        <div class="p-4 border-b xl:border-b-0 xl:border-r border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">中央値</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {work.statistics.median}
                            </div>
                        </div>
                        <div class="p-4 border-b md:border-b-0 md:border-r border-border-primary min-w-0">
                            <div class="text-caption text-text-tertiary">データ数</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {work.statistics.count}
                            </div>
                        </div>
                        <div class="p-4 min-w-0">
                            <div class="text-caption text-text-tertiary">属性</div>
                            <div class="text-body2 text-text-secondary font-medium truncate">
                                {element.isNukige ? "抜きゲー" : "通常"}
                            </div>
                        </div>
                    </div>
                </section>
            </div>

            <section
                class="rounded-lg border border-border-primary bg-bg-primary/24 backdrop-blur-md shadow-sm p-4 lg:p-5 min-w-0"
                aria-labelledby="work-creators-title"
            >
                <h2 id="work-creators-title" class="text-h3 text-text-primary font-bold mb-4">
                    詳細情報
                </h2>
                <Detail {work} />
            </section>
        </div>
    {:else if page === "record"}
        <section
            id="work-record"
            class="rounded-lg border border-border-primary bg-bg-primary/30 backdrop-blur-md shadow-sm p-4 lg:p-5"
            aria-labelledby="work-record-title"
        >
            <div class="flex items-center gap-2 mb-4">
                <div class="i-material-symbols-history-rounded w-5 h-5 color-ui-tertiary" />
                <h2 id="work-record-title" class="text-h3 text-text-primary font-bold">
                    記録
                </h2>
            </div>
            <div class="mb-6 flex flex-wrap items-center gap-x-10 gap-y-5">
                {#each recordRows as row (row.label)}
                    <div class="flex min-w-[10rem] items-center gap-3 rounded-lg border border-border-primary bg-bg-secondary/10 backdrop-blur-sm px-3 py-2">
                        <div class="{row.icon} h-7 w-7 shrink-0 color-ui-tertiary" />
                        <div class="min-w-0">
                            <div class="text-caption text-text-tertiary">
                                {row.label}
                            </div>
                            <div class="mt-1 max-w-44 truncate text-body2 font-semibold text-text-primary">
                                {row.value}
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
            <div class="rounded-lg border border-border-primary bg-bg-secondary/5 backdrop-blur-sm p-4">
                <div class="text-caption text-text-tertiary">コミュニティ統計</div>
                <div class="mt-3 grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <div class="min-w-0">
                        <div class="text-caption text-text-tertiary">平均プレイ時間</div>
                        <div class="mt-1 rounded-md bg-bg-secondary/5 px-2.5 py-2 text-body2 text-text-primary font-semibold truncate backdrop-blur-sm">
                            {work.statistics.playTime}
                        </div>
                    </div>
                    <div class="min-w-0">
                        <div class="text-caption text-text-tertiary">中央値</div>
                        <div class="mt-1 rounded-md bg-bg-secondary/5 px-2.5 py-2 text-body2 text-text-primary font-semibold truncate backdrop-blur-sm">
                            {work.statistics.median}
                        </div>
                    </div>
                    <div class="min-w-0">
                        <div class="text-caption text-text-tertiary">データ数</div>
                        <div class="mt-1 rounded-md bg-bg-secondary/5 px-2.5 py-2 text-body2 text-text-primary font-semibold truncate backdrop-blur-sm">
                            {work.statistics.count}
                        </div>
                    </div>
                </div>
            </div>
        </section>
    {:else if page === "memo"}
        <section
            id="work-memo"
            class="rounded-lg border border-border-primary/40 bg-bg-primary/20 backdrop-blur-lg shadow-xl p-4 lg:p-5 transition-all duration-300"
            aria-labelledby="work-memo-title"
        >
            <div class="flex items-start gap-3">
                <div class="i-material-symbols-drive-file-rename-outline w-6 h-6 color-ui-tertiary shrink-0 mt-1" />
                <div class="min-w-0 flex-1">
                    <h2 id="work-memo-title" class="text-h3 text-text-primary font-bold">
                        Memo
                    </h2>
                    <a
                        href={`/memos/${work.id}?gamename=${encodeURIComponent(element.gamename)}`}
                        use:link
                        class="mt-4 inline-flex items-center gap-2 rounded border border-border-primary/40 bg-bg-button/20 backdrop-blur-md px-3 py-2 text-body2 text-text-primary hover:bg-bg-button/45 transition-colors focus-visible:ring-2 focus-visible:ring-accent-accent"
                    >
                        <div class="i-material-symbols-open-in-new-rounded w-4 h-4" />
                        メモを開く
                    </a>

                    <div class="mt-6 border-t border-border-primary/50 pt-5">
                        <h3 class="text-body2 font-semibold text-text-primary mb-3">
                            メモ
                        </h3>
                        {#key work.id}
                            <textarea use:mde />
                        {/key}
                    </div>
                </div>
            </div>
        </section>
    {:else if page === "screenshots"}
        <section
            id="work-screenshots"
            class="rounded-lg border border-border-primary bg-bg-primary/72 shadow-sm p-4 lg:p-5"
            aria-label="スクリーンショット"
        >
            <ScreenshotGallery gameId={work.id} />
        </section>
    {/if}
</div>
{/if}
