import type { CollectionElement } from "@/lib/types";
import { writable, type Readable, derived } from "svelte/store";

export const useVirtualScrollerMasonry = (
  elements: Readable<CollectionElement[]>,
  setVirtualHeight: (v: number) => void,
  contentsWidth: Readable<number>,
  contentsScrollY: Readable<number>,
  containerHeight: Readable<number>
) => {
  const minItemWidth = 16 * 16;
  const itemGap = 16;
  const placeholderHeight = 16 * 8;

  type Cell = {
    top: number;
    left: number;
    width: number;
    height: number;
    element: CollectionElement;
  };
  type Layout = Cell[][];

  const buffer = 5;
  const beamWidth = 50;

  // 戻り値を [最長列の高さ, 最大差] のペアに変更
  function evaluateGreedy(
    base: Layout,
    remaining: CollectionElement[],
    itemWidth: number
  ): [number, number] { // 戻り値の型を変更
    const cols = base.map(col => col.slice());
    for (const ele of remaining) {
      const h = ele.thumbnailWidth && ele.thumbnailHeight && ele.thumbnailWidth > 0
        ? Math.floor((itemWidth / ele.thumbnailWidth) * ele.thumbnailHeight)
        : placeholderHeight;
      const bottoms = cols.map(col =>
        col.length > 0
          ? col[col.length - 1].top + col[col.length - 1].height
          : 0
      );
      const minBottom = Math.min(...bottoms.filter(b => !isNaN(b))); // NaNを除外
      const idx = !isNaN(minBottom) ? bottoms.indexOf(minBottom) : 0;
      // top の計算を修正: 以前の高さが 0 でも gap を追加しないように
      const top = bottoms[idx] > 0 ? bottoms[idx] + itemGap : 0;
      cols[idx].push({ top, left: idx * (itemWidth + itemGap), width: itemWidth, height: h, element: ele });
    }
    // 配置後の各列の高さを計算
    const finalHeights = cols.map(col =>
      col.length > 0 ? col[col.length - 1].top + col[col.length - 1].height : 0
    ).filter(h => !isNaN(h)); // NaNを除外

    if (finalHeights.length === 0) {
        return [0, 0]; // 要素がない場合は高さ0、差0
    }

    const maxHeight = Math.max(...finalHeights);
    const minHeight = Math.min(...finalHeights);
    const diff = maxHeight - minHeight;

    return [maxHeight, diff]; // 評価ペアを返す
  }

  const calculateLayouts = (
    elements: CollectionElement[],
    containerWidth: number
  ): { layout: Layout; columns: number; itemWidth: number } => {
    if (!containerWidth || elements.length === 0) return { layout: [], columns: 0, itemWidth: 0 };
    const itemNumPerRow = Math.max(1, Math.floor((containerWidth + itemGap) / (minItemWidth + itemGap)));
    const itemWidth = Math.floor((containerWidth - itemGap * (itemNumPerRow - 1)) / itemNumPerRow);

    // beams の score の型を [number, number] に変更
    type Beam = { layout: Layout; score: [number, number] };

    const initialLayout: Layout = Array.from({ length: itemNumPerRow }, () => []);
    // 初期ビームのスコアも新しい評価関数で計算
    let beams: Beam[] = [
      { layout: initialLayout, score: evaluateGreedy(initialLayout, elements, itemWidth) }
    ];

    for (const [idx, ele] of elements.entries()) {
      const newBeams: Beam[] = []; // 型を Beam[] に
      const h = ele.thumbnailWidth && ele.thumbnailHeight && ele.thumbnailWidth > 0
        ? Math.floor((itemWidth / ele.thumbnailWidth) * ele.thumbnailHeight)
        : placeholderHeight;

      for (const beam of beams) {
        const currentHeights = beam.layout.map(col =>
          col.length > 0
            ? col[col.length - 1].top + col[col.length - 1].height
            : 0
        );

        // --- 列数の半分を閾値とする ---
        const numColumnsForThreshold = beam.layout.length;
        const PLACEMENT_RANK_THRESHOLD = Math.ceil(numColumnsForThreshold / 2);

        for (let colIdx = 0; colIdx < beam.layout.length; colIdx++) {
          let isPlacementValid = true;
          const numColumns = beam.layout.length;

          if (numColumns > 1) {
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

          if (!isPlacementValid) {
              continue;
          }

          const nextLayout = beam.layout.map(col => col.slice());
          const currentBottom = currentHeights[colIdx];
          // top の計算を修正: 以前の高さが 0 でも gap を追加しないように
          const top = currentBottom > 0 ? currentBottom + itemGap : 0;
          nextLayout[colIdx].push({ top, left: colIdx * (itemWidth + itemGap), width: itemWidth, height: h, element: ele });
          const remaining = elements.slice(idx + 1);
          // 新しい評価関数でスコアを取得
          const score = evaluateGreedy(nextLayout, remaining, itemWidth);
          newBeams.push({ layout: nextLayout, score });
        } // end for colIdx
      } // end for beam

      if (newBeams.length === 0) {
          // フォールバック処理 (変更なし、ただし score はペアになる)
          if (beams.length > 0) {
              const fallbackLayout = beams[0].layout.map(col => col.slice());
              const fallbackHeights = fallbackLayout.map(col =>
                  col.length > 0 ? col[col.length - 1].top + col[col.length - 1].height : 0
              );
               const validHeights = fallbackHeights.filter(hh => !isNaN(hh));
               const minFallbackHeight = validHeights.length > 0 ? Math.min(...validHeights) : 0;
               const fallbackColIdx = fallbackHeights.indexOf(minFallbackHeight);

               const fallbackBottom = fallbackHeights[fallbackColIdx];
               // top の計算を修正
               const fallbackTop = fallbackBottom > 0 ? fallbackBottom + itemGap : 0;
               fallbackLayout[fallbackColIdx].push({ top: fallbackTop, left: fallbackColIdx * (itemWidth + itemGap), width: itemWidth, height: h, element: ele });
               // スコアもペアで計算
               const fallbackScore = evaluateGreedy(fallbackLayout, elements.slice(idx + 1), itemWidth);
               beams = [{ layout: fallbackLayout, score: fallbackScore }];
          } else {
              beams = [];
              break;
          }
      } else {
          // ビームのソート処理を新しいスコアペアに基づいて変更
          newBeams.sort((a, b) => {
              // 1. 最長列の長さで比較
              if (a.score[0] !== b.score[0]) {
                return a.score[0] - b.score[0]; // 短い方が良い
              }
              // 2. 最長列の長さが同じなら、差で比較
              return a.score[1] - b.score[1]; // 差が小さい方が良い
          });
          beams = newBeams.slice(0, beamWidth);
      }
    } // end for elements

    const bestLayout = beams.length > 0 ? beams[0].layout : initialLayout;
    return { layout: bestLayout, columns: itemNumPerRow, itemWidth };
  };

  let prevColumns = 0; // 前回の列数
  let prevLayout: Layout = []; // 前回のレイアウト（配置順序と要素を保持）
  let prevElementIds: number[] = [];
  
  // 要素リストが変更されたかチェックする関数
  const didElementsChange = (currentElements: CollectionElement[], previousIds: number[]): boolean => {
    const currentIds = currentElements.map(el => el.id);
    if (currentIds.length !== previousIds.length) {
      return true; // 要素数が変わったら変更あり
    }
    // 要素数が同じなら、順序を含めてIDが一致するかチェック
    for (let i = 0; i < currentIds.length; i++) {
      if (currentIds[i] !== previousIds[i]) {
        return true; // IDまたは順序が異なれば変更あり
      }
    }
    return false; // ここまで来たら変更なし
  };

  const layouts = derived<
    [typeof elements, typeof contentsWidth],
    Layout
  >(
    [elements, contentsWidth],
    ([$elements, $contentsWidth], set) => {
      // コンテナ幅がない、または要素がない場合は空レイアウトをセットして終了
      if (!$contentsWidth || $elements.length === 0) {
        prevColumns = 0;
        prevLayout = [];
        prevElementIds = [];
        set([]);
        return;
      }

      // 1. 現在のコンテナ幅から新しい列数とアイテム幅を計算
      const newColumns = Math.max(1, Math.floor(($contentsWidth + itemGap) / (minItemWidth + itemGap)));
      const newItemWidth = Math.floor(($contentsWidth - itemGap * (newColumns - 1)) / newColumns);

      let resultLayout: Layout;
      
      const elementsChanged = didElementsChange($elements, prevElementIds);

      // 2. 列数が前回から確実に変わったか、または前回のレイアウトが空か？
      if (newColumns !== prevColumns || elementsChanged || prevLayout.flat().length === 0) {
        // 列数が変わった -> ビームサーチでレイアウトを最初から再計算
        // calculateLayouts内で beamWidth=50 が使用される
        const { layout } = calculateLayouts($elements, $contentsWidth);
        resultLayout = layout;
        // console.log(`Layout recalculated (beam search): ${newColumns} columns`);
      } else {
        // 列数が同じ -> 前回の配置順序 (prevLayout) を維持し、サイズと位置のみ更新
        // console.log(`Layout updated (size/position only): ${newColumns} columns`);
        const updatedLayout: Layout = prevLayout.map((col, colIdx) => {
          let currentTop = 0;
          // map の結果が undefined にならないように型を明示、または filter(Boolean)
          const updatedCol: Cell[] = col.map((cell): Cell | undefined => {
            // 要素データが見つからないケースはスキップ（基本的には起こらないはず）
            if (!cell || !cell.element) return undefined;

            // 新しいアイテム幅に基づいて高さを再計算
            const h = cell.element.thumbnailWidth && cell.element.thumbnailHeight && cell.element.thumbnailWidth > 0
              ? Math.floor((newItemWidth / cell.element.thumbnailWidth) * cell.element.thumbnailHeight)
              : placeholderHeight;

            const top = currentTop;
            const updatedCell: Cell = {
              top,
              left: colIdx * (newItemWidth + itemGap), // 新しい位置
              width: newItemWidth, // 新しい幅
              height: h, // 新しい高さ
              element: cell.element, // 要素は同じ
            };
            currentTop = top + h + itemGap;
            return updatedCell;
          }).filter((cell): cell is Cell => cell !== undefined); // undefinedを除外
          return updatedCol;
        });
        resultLayout = updatedLayout;
      }

      // 次回比較のために現在の状態を保存
      prevColumns = newColumns;
      prevLayout = resultLayout; // 更新/再計算されたレイアウトを保持
      prevElementIds = $elements.map(el => el.id);

      // 計算結果をセットしてストアを更新
      set(resultLayout);
    },
    [] // derived の初期値（空のレイアウト）
  );

  layouts.subscribe(cols => {
    const heights = cols.map(col =>
      col.length > 0 ? col[col.length - 1].top + col[col.length - 1].height : 0
    );
    setVirtualHeight(Math.max(...heights));
  });

  const calculateVisibleLayouts = (
    cols: Layout,
    scrollTop: number,
    contentsHeight: number
  ) => {
    const visible: Cell[] = [];
    cols.forEach(col => {
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
      set(calculateVisibleLayouts($layouts, $contentsScrollY, $masonryContainerHeight));
    }
  );

  return { visibleLayouts };
};
