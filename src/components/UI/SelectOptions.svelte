<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import { useFilter } from "@/lib/filter";
  import { type Option } from "@/lib/trieFilter";
  import { createWritable } from "@/lib/utils";
  import { createEventDispatcher, onMount } from "svelte";
  import { writable } from "svelte/store";

  export let options: Option<string | number>[];
  export let title: string | undefined = undefined;
  export let enableFilter: boolean = false;
  export let showSelectedCheck = true;
  export let filterPlaceholder = "";
  export let bottomCreateButtonText = "";
  export let value: string | number;

  const [writableOptions, getOptions] = createWritable(options);
  $: {
    writableOptions.set(options);
  }

  const localQuery = writable("");
  const { filtered } = useFilter(localQuery, writableOptions, getOptions);

  const dispatcher = createEventDispatcher<{
    select: { value: string | number };
    create: {};
    close: {};
  }>();

  // 選択されているパス（各階層の現在のアクティブな項目）
  let activePath: Option<string | number>[] = [];

  $: columns = calculateColumns($filtered, activePath);

  function calculateColumns(
    rootOptions: Option<string | number>[],
    path: Option<string | number>[],
  ) {
    const cols = [rootOptions];
    for (let i = 0; i < path.length; i++) {
      const itemInPath = path[i];
      // 現在のフィルタリングされたオプションの中に、パスに含まれるアイテムがあるか探す
      // (フィルタリングでアイテムが消えてもパスには残っている可能性があるため)
      const item = findOptionInList(
        i === 0 ? rootOptions : cols[i],
        itemInPath?.value,
      );
      if (item && item.children && item.children.length > 0) {
        cols.push(item.children);
      } else {
        break;
      }
    }
    return cols;
  }

  function findOptionInList(
    list: Option<string | number>[],
    val: string | number | undefined,
  ) {
    if (!val) return null;
    return list.find((o) => o.value === val) || null;
  }

  function handleItemClick(option: Option<string | number>, depth: number) {
    const isAlreadyInPath = activePath[depth]?.value === option.value;

    if (isAlreadyInPath && option.children && option.children.length > 0) {
      // 既に開いているフォルダをクリックした場合は、自分自身を含めてそれより右をすべて閉じる
      activePath = activePath.slice(0, depth);
    } else {
      // 新しくクリックした場合はパスを更新
      const newPath = activePath.slice(0, depth);
      newPath[depth] = option;
      activePath = newPath;

      if (!option.children || option.children.length === 0) {
        // 最終要素なら選択を確定
        value = option.value;
        dispatcher("select", { value: option.value });
        $localQuery = "";
        dispatcher("close");
      }
    }
  }

  function isItemInPath(option: Option<string | number>, depth: number) {
    return activePath[depth]?.value === option.value;
  }
</script>

<div class="flex flex-row gap-2 overflow-visible p-1 items-start">
  {#each columns as columnOptions, depth}
    <div
      class="flex flex-(col) min-w-48 max-w-64 bg-bg-secondary rounded shadow-xl border border-(border-primary solid) overflow-hidden"
    >
      {#if depth === 0 && title}
        <div
          class="p-(x-4 y-2) border-b border-(border-primary solid) text-(body2 text-primary) font-bold truncate bg-bg-tertiary"
        >
          {title}
        </div>
      {/if}
      {#if depth === 0 && enableFilter}
        <div class="p-2 border-b border-(border-primary solid)">
          <Input
            bind:value={$localQuery}
            placeholder={filterPlaceholder}
            autofocus
          />
        </div>
      {/if}
      <div class="overflow-y-auto max-h-96 flex-grow flex flex-col">
        {#each columnOptions as option (option.value)}
          <button
            class={`p-(x-4 y-2) border-b border-(border-primary solid) last:border-b-0
                    hover:bg-bg-tertiary w-full flex items-center gap-2 transition-all cursor-pointer text-left
                    ${isItemInPath(option, depth) ? "bg-bg-tertiary" : "bg-transparent"}`}
            on:click={() => handleItemClick(option, depth)}
          >
            {#if showSelectedCheck && value === option.value}
              <div
                class="h-5 w-5 color-text-primary flex-shrink-0 i-material-symbols-check-small-rounded"
              />
            {:else if showSelectedCheck}
              <div class="h-5 w-5 flex-shrink-0" />
            {/if}
            <div
              class="text-(body2 text-primary) font-medium truncate flex-grow"
            >
              {option.label}
            </div>
            {#if option.children && option.children.length > 0}
              <div
                class="i-material-symbols-chevron-right-rounded h-5 w-5 color-text-primary flex-shrink-0 ml-auto transition-transform"
                class:rotate-90={isItemInPath(option, depth)}
              />
            {/if}
          </button>
        {/each}
      </div>
      {#if depth === 0 && bottomCreateButtonText}
        <button
          class="bg-transparent hover:bg-bg-tertiary transition-all w-full p-(l-4 r-5 y-2) flex items-center border-t border-(border-primary solid)"
          on:click={() => dispatcher("create")}
        >
          <div class="w-5 h-5 i-iconoir-plus color-text-primary" />
          <div
            class="text-(text-primary body2 left) font-bold whitespace-nowrap"
          >
            {bottomCreateButtonText}
          </div>
        </button>
      {/if}
    </div>
  {/each}
</div>
