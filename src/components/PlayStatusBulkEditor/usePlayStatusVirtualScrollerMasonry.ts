import type { CollectionElement } from "@/lib/types";
import { writable, type Readable, derived } from "svelte/store";

interface MasonryOptions {
  minItemWidth: number;
  itemGap: number;
  tileInternalPadding: number;
  titleAreaHeight?: number;
  placeholderAspectRatio?: number;
}

export const usePlayStatusVirtualScrollerMasonry = (
  elements: Readable<CollectionElement[]>,
  setVirtualHeight: (v: number) => void,
  contentsWidth: Readable<number>,
  contentsScrollY: Readable<number>,
  containerHeight: Readable<number>,
  options: MasonryOptions
) => {
  const {
    minItemWidth,
    itemGap,
    tileInternalPadding,
    titleAreaHeight = 40,
    placeholderAspectRatio = 4 / 3,
  } = options;

  type Cell = {
    top: number;
    left: number;
    width: number;
    height: number;
    element: CollectionElement; // ★常に最新の要素データを持つようにする
    imgDisplayWidth: number;
    imgDisplayHeight: number;
  };
  type Layout = Cell[][];

  const buffer = 5;
  const beamWidth = 50;

  function calculateImageHeight(element: CollectionElement, imageWidth: number): number {
    if (element.thumbnailWidth && element.thumbnailHeight && element.thumbnailWidth > 0) {
      const aspectRatio = element.thumbnailHeight / element.thumbnailWidth;
      return Math.floor(imageWidth * aspectRatio);
    }
    return Math.floor(imageWidth / placeholderAspectRatio);
  }

  function evaluateGreedy(
    base: Layout,
    remaining: CollectionElement[],
    columnWidth: number
  ): [number, number] {
    const cols = base.map(col => col.slice());
    const actualImgDisplayWidth = Math.max(1, columnWidth - (tileInternalPadding * 2));

    for (const ele of remaining) {
      const imgH = calculateImageHeight(ele, actualImgDisplayWidth);
      const h = imgH + titleAreaHeight;

      const bottoms = cols.map(col =>
        col.length > 0
          ? col[col.length - 1].top + col[col.length - 1].height
          : 0
      );
      const minBottom = Math.min(...bottoms.filter(b => !isNaN(b) && isFinite(b)));
      const idx = !isNaN(minBottom) && isFinite(minBottom) ? bottoms.indexOf(minBottom) : 0;
      const top = bottoms[idx] > 0 ? bottoms[idx] + itemGap : 0;

      cols[idx].push({
        top,
        left: idx * (columnWidth + itemGap),
        width: columnWidth,
        height: h,
        element: ele, // evaluateGreedyでは渡された最新のelementを使う
        imgDisplayWidth: actualImgDisplayWidth,
        imgDisplayHeight: imgH,
      });
    }
    const finalHeights = cols.map(col =>
      col.length > 0 ? col[col.length - 1].top + col[col.length - 1].height : 0
    ).filter(h => !isNaN(h) && isFinite(h));

    if (finalHeights.length === 0) return [0, 0];
    const maxHeight = Math.max(...finalHeights);
    const minHeight = Math.min(...finalHeights);
    const diff = maxHeight - minHeight;
    return [maxHeight, diff];
  }

  const calculateLayoutsWithBeamSearch = (
    currentElements: CollectionElement[], // ★引数名を変更
    containerWidth: number
  ): { layout: Layout; columns: number; itemWidth: number } => {
    if (!containerWidth || currentElements.length === 0) return { layout: [], columns: 0, itemWidth: 0 };

    const numCols = Math.max(1, Math.floor((containerWidth + itemGap) / (minItemWidth + itemGap)));
    const colWidth = Math.floor((containerWidth - itemGap * (numCols - 1)) / numCols);
    const actualImgDisplayWidth = Math.max(1, colWidth - (tileInternalPadding * 2));

    type Beam = { layout: Layout; score: [number, number] };
    const initialLayout: Layout = Array.from({ length: numCols }, () => []);
    let beams: Beam[] = [
      { layout: initialLayout, score: evaluateGreedy(initialLayout, currentElements, colWidth) }
    ];

    for (const [idx, ele] of currentElements.entries()) { // ★currentElements を使う
      const newBeams: Beam[] = [];
      const imgH = calculateImageHeight(ele, actualImgDisplayWidth);
      const h = imgH + titleAreaHeight;

      for (const beam of beams) {
        const currentHeights = beam.layout.map(col =>
          col.length > 0
            ? col[col.length - 1].top + col[col.length - 1].height
            : 0
        );
        const numColumnsForThreshold = beam.layout.length;
        const PLACEMENT_RANK_THRESHOLD = Math.ceil(numColumnsForThreshold / 2);

        for (let colIdx = 0; colIdx < numCols; colIdx++) {
          let isPlacementValid = true;
          if (numCols > 1) {
              const currentCol = beam.layout[colIdx];
              const hypotheticalHeight = currentCol.length > 0
                  ? (currentCol[currentCol.length - 1].top > 0 ? currentCol[currentCol.length - 1].top - itemGap : 0)
                  : 0;
              const allHeightsForCheck = currentHeights.map((height, i) =>
                  i === colIdx ? hypotheticalHeight : height
              );
              const sortedHeights = [...allHeightsForCheck].sort((a, b) => b - a);
              const rankIndex = sortedHeights.findIndex(height => height === hypotheticalHeight);
              if (hypotheticalHeight > 0 && rankIndex !== -1 && rankIndex < PLACEMENT_RANK_THRESHOLD) {
                  isPlacementValid = false;
              }
          }
          if (!isPlacementValid) continue;

          const nextLayout = beam.layout.map(colData => colData.slice());
          const currentColumnBottom = currentHeights[colIdx];
          const top = currentColumnBottom > 0 ? currentColumnBottom + itemGap : 0;
          nextLayout[colIdx].push({
            top,
            left: colIdx * (colWidth + itemGap),
            width: colWidth,
            height: h,
            element: ele, // ★currentElements の ele を使う
            imgDisplayWidth: actualImgDisplayWidth,
            imgDisplayHeight: imgH,
          });
          const remaining = currentElements.slice(idx + 1); // ★currentElements を使う
          const score = evaluateGreedy(nextLayout, remaining, colWidth);
          newBeams.push({ layout: nextLayout, score });
        }
      }
      if (newBeams.length === 0) {
          if (beams.length > 0) {
              const fallbackLayout = beams[0].layout.map(colData => colData.slice());
              const fallbackHeights = fallbackLayout.map(colData =>
                  colData.length > 0 ? colData[colData.length - 1].top + colData[colData.length - 1].height : 0
              );
               const validHeights = fallbackHeights.filter(hh => !isNaN(hh) && isFinite(hh));
               const minFallbackHeight = validHeights.length > 0 ? Math.min(...validHeights) : 0;
               const fallbackColIdx = fallbackHeights.indexOf(minFallbackHeight);
               const fallbackBottom = fallbackHeights[fallbackColIdx];
               const fallbackTop = fallbackBottom > 0 ? fallbackBottom + itemGap : 0;
               fallbackLayout[fallbackColIdx].push({
                 top: fallbackTop,
                 left: fallbackColIdx * (colWidth + itemGap),
                 width: colWidth,
                 height: h,
                 element: ele, // ★currentElements の ele を使う
                 imgDisplayWidth: actualImgDisplayWidth,
                 imgDisplayHeight: imgH,
                });
               const fallbackScore = evaluateGreedy(fallbackLayout, currentElements.slice(idx + 1), colWidth); // ★
               beams = [{ layout: fallbackLayout, score: fallbackScore }];
          } else { beams = []; break; }
      } else {
          newBeams.sort((a, b) => {
              if (a.score[0] !== b.score[0]) return a.score[0] - b.score[0];
              return a.score[1] - b.score[1];
          });
          beams = newBeams.slice(0, beamWidth);
      }
    }
    const bestLayout = beams.length > 0 ? beams[0].layout : initialLayout;
    return { layout: bestLayout, columns: numCols, itemWidth: colWidth };
  };

  let prevColumns = 0;
  let prevLayout: Layout = [];
  let prevElementContents: CollectionElement[] = []; // ★変更: 要素の内容を比較するためにIDだけでなく要素全体を保持（またはplayStatusだけでも良い）
  let prevContainerWidth = 0;

  // ★変更: 要素の内容（特にplayStatus）が変わったかもチェックする
  const didElementsDataChange = (currentElements: CollectionElement[], previousElements: CollectionElement[]): boolean => {
    if (currentElements.length !== previousElements.length) return true;
    for (let i = 0; i < currentElements.length; i++) {
      if (currentElements[i].id !== previousElements[i].id ||
          currentElements[i].playStatus !== previousElements[i].playStatus ||
          currentElements[i].thumbnail !== previousElements[i].thumbnail // 他に影響するプロパティがあれば追加
      ) {
        return true;
      }
    }
    return false;
  };

  const layouts = derived<
    [typeof elements, typeof contentsWidth],
    Layout
  >(
    [elements, contentsWidth],
    ([$elements, $contentsWidth], set) => {
      if (!$contentsWidth || $elements.length === 0) {
        prevColumns = 0; prevLayout = []; prevElementContents = []; prevContainerWidth = 0;
        set([]); return;
      }

      const newNumColumns = Math.max(1, Math.floor(($contentsWidth + itemGap) / (minItemWidth + itemGap)));
      const newColumnWidth = Math.floor(($contentsWidth - itemGap * (newNumColumns - 1)) / newNumColumns);
      const newActualImgDisplayWidth = Math.max(1, newColumnWidth - (tileInternalPadding * 2));

      let resultLayout: Layout;
      const elementsDataChanged = didElementsDataChange($elements, prevElementContents); // ★変更: 内容の変更をチェック

      if (newNumColumns !== prevColumns || elementsDataChanged || prevLayout.flat().length === 0 ) {
        const { layout } = calculateLayoutsWithBeamSearch($elements, $contentsWidth);
        resultLayout = layout;
      } else {
        // 列数も要素の内容・順序も変わらない場合 (コンテナ幅だけが変わったなど)
        // => 前回の配置構造 (prevLayout) を元に、サイズと位置のみ更新
        // このパスでは、各cell.element は $elements の最新のものを参照するようにする
        const elementsMap = new Map($elements.map(el => [el.id, el])); // 最新の要素データをIDで引けるようにする

        resultLayout = prevLayout.map((originalColInSection, colIdx) => {
          let currentTopInCol = 0;
          return originalColInSection.map((cell) => {
            const latestElement = elementsMap.get(cell.element.id) || cell.element; // ★最新の要素データを取得
            const imgH = calculateImageHeight(latestElement, newActualImgDisplayWidth);
            const h = imgH + titleAreaHeight;
            const top = currentTopInCol;
            currentTopInCol += h + itemGap;

            return {
              // ...cell, // ここで ...cell を展開すると古いelementが使われる可能性
              top,
              left: colIdx * (newColumnWidth + itemGap),
              width: newColumnWidth,
              height: h,
              element: latestElement, // ★最新の要素データを使用
              imgDisplayWidth: newActualImgDisplayWidth,
              imgDisplayHeight: imgH,
            };
          });
        });
      }
      prevColumns = newNumColumns;
      prevLayout = resultLayout;
      prevElementContents = $elements.map(el => ({...el})); // ★要素の内容をディープコピーして保持
      prevContainerWidth = $contentsWidth;
      set(resultLayout);
    },
    []
  );

  layouts.subscribe(cols => {
    if (!cols || cols.length === 0) {
        setVirtualHeight(0);
        return;
    }
    const heights = cols.map(col =>
      col.length > 0 ? col[col.length - 1].top + col[col.length - 1].height : 0
    ).filter(h => !isNaN(h) && isFinite(h));
    setVirtualHeight(heights.length > 0 ? Math.max(...heights) : 0);
  });

  layouts.subscribe(cols => {
    const heights = cols.map(col =>
      col.length > 0 ? col[col.length - 1].top + col[col.length - 1].height : 0
    );
    setVirtualHeight(heights.length > 0 ? Math.max(...heights) : 0);
  });

  const calculateVisibleLayouts = (
    cols: Layout,
    scrollTop: number,
    contentsHeight: number
  ) => {
    const visible: Cell[] = [];
    cols.forEach(col => {
      if (!col) return; // col が undefined の場合をスキップ
      const first = col.findIndex(cell => cell.top + cell.height >= scrollTop);
      let last = col.findIndex(cell => cell.top >= scrollTop + contentsHeight);
      if (first === -1) return;
      if (last === -1) last = col.length - 1;
      const start = Math.max(first - buffer, 0);
      const end = Math.min(last + buffer, col.length - 1);
      visible.push(...col.slice(start, end + 1));
    });
    return visible;
  };

  const visibleLayouts = derived<
    [typeof layouts, typeof contentsScrollY, typeof containerHeight],
    Cell[]
  >(
    [layouts, contentsScrollY, containerHeight],
    ([$layouts, $contentsScrollY, $masonryContainerHeight], set) => {
      if (!$layouts) { // $layouts が undefined の場合を考慮
        set([]);
        return;
      }
      set(calculateVisibleLayouts($layouts, $contentsScrollY, $masonryContainerHeight));
    },
    [] // 初期値
  );

  return { visibleLayouts };
};
