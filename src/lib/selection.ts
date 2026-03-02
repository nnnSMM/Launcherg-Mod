export const selectAll = (
    displayedItems: { id: number }[],
    currentSelection: Set<number>
): Set<number> => {
    const newSelection = new Set(currentSelection);
    for (const item of displayedItems) {
        newSelection.add(item.id);
    }
    return newSelection;
};

export const deselectAll = (
    displayedItems: { id: number }[],
    currentSelection: Set<number>
): Set<number> => {
    const newSelection = new Set(currentSelection);
    for (const item of displayedItems) {
        newSelection.delete(item.id);
    }
    return newSelection;
};

export const toggleAll = (
    displayedItems: { id: number }[],
    currentSelection: Set<number>
): Set<number> => {
    const newSelection = new Set(currentSelection);
    for (const item of displayedItems) {
        if (newSelection.has(item.id)) {
            newSelection.delete(item.id);
        } else {
            newSelection.add(item.id);
        }
    }
    return newSelection;
};
