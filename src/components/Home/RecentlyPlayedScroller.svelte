<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from "svelte";

  let scrollableElement: HTMLDivElement;

  // `Home.svelte` の矢印ボタンのために `scrollBy` をエクスポートします
  export const scrollBy = (options: ScrollToOptions) => {
    scrollableElement?.scrollBy(options);
  };

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>();

  // ドラッグ＆スクロールの状態管理
  let isDown = false;
  let startX: number;
  let scrollLeft: number;
  let isDragging = false; // ドラッグ操作かを判定するフラグ
  const DRAG_THRESHOLD = 5; // ドラッグと見なす最小移動ピクセル数

  const handleMouseDown = (e: MouseEvent) => {
    // テキスト選択や画像のドラッグなどのデフォルト動作を防止
    e.preventDefault();
    isDown = true;
    isDragging = false; // ドラッグ状態をリセット
    if (scrollableElement) {
      scrollableElement.classList.add("active");
      startX = e.pageX - scrollableElement.offsetLeft;
      scrollLeft = scrollableElement.scrollLeft;
    }

    // マウスの動きをウィンドウ全体で捕捉するためのリスナーを追加
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  };

  const handleMouseUp = () => {
    isDown = false;
    if (scrollableElement) {
      scrollableElement.classList.remove("active");
    }
    // グローバルリスナーをクリーンアップ
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);

    // clickイベントがisDraggingフラグを読んだ後にリセットするため、
    // イベントキューの最後にこの処理をスケジュールする
    setTimeout(() => {
      isDragging = false;
    }, 0);
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDown || !scrollableElement) return;
    e.preventDefault();

    const x = e.pageX - scrollableElement.offsetLeft;
    const walk = x - startX;

    // DRAG_THRESHOLDを超えてマウスが動いたら、ドラッグ操作と見なす
    if (!isDragging && Math.abs(walk) > DRAG_THRESHOLD) {
      isDragging = true;
    }

    // 1:1の自然なスクロール速度
    scrollableElement.scrollLeft = scrollLeft - walk;
  };

  // ドラッグ操作だった場合、子要素のクリックイベントをキャンセルする
  const handleClick = (e: MouseEvent) => {
    if (isDragging) {
      e.stopPropagation();
      e.preventDefault();
    }
  };

  // 親コンポーネント(Home.svelte)のためにスクロールイベントをディスパッチ
  const onScroll = (e: Event) => {
    dispatcher("scroll", { event: e });
  };

  onMount(() => {
    if (scrollableElement) {
      scrollableElement.addEventListener('scroll', onScroll);
    }
  });

  onDestroy(() => {
    // コンポーネント破棄時にリスナーを確実にクリーンアップ
    if (scrollableElement) {
        scrollableElement.removeEventListener('scroll', onScroll);
    }
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);
  });
</script>

<style>
  .scroller {
    overflow-x: auto;
    overflow-y: hidden; /* 縦スクロールを完全に禁止 */
    cursor: grab;
    user-select: none; /* ドラッグ中のテキスト選択を防止 */
    /* デフォルトのスクロールバーを非表示にする */
    scrollbar-width: none; /* Firefox */
    -ms-overflow-style: none;  /* Internet Explorer 10+ */
  }
  .scroller::-webkit-scrollbar { /* WebKit */
    display: none;
  }
  .scroller.active {
    cursor: grabbing;
  }
</style>

<div
  bind:this={scrollableElement}
  class="scroller"
  on:mousedown={handleMouseDown}
  on:click|capture={handleClick}
>
  <slot />
</div>