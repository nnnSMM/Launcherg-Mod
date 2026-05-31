export const isNearTopEdge = (clientY: number, threshold = 5): boolean => {
    return clientY < threshold;
};

export const isNearBottomEdge = (clientY: number, windowHeight: number, threshold = 5): boolean => {
    return clientY >= windowHeight - threshold;
};

export const isInsideTopEdgeArea = (clientY: number, height: number): boolean => {
    return clientY < height;
};
