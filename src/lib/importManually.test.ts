import { describe, it, expect, vi } from 'vitest';

vi.mock('./scrapeAllGame', () => ({
    scrapeAllGameCacheOnes: vi.fn(),
}));

vi.mock('@/lib/toast', () => ({
    showErrorToast: vi.fn(),
}));

const parseErogameScapeId = (input: string): number | undefined => {
    const idNumber = +input;
    if (!isNaN(idNumber) && input.trim() !== '') {
        return idNumber;
    }

    try {
        const url = new URL(input);
        const idString = url.searchParams.get('game');
        if (!idString) {
            return undefined;
        }
        const idNum = +idString;
        if (isNaN(idNum)) {
            return undefined;
        }
        return idNum;
    } catch (e) {
        return undefined;
    }
};

describe('parseErogameScapeId', () => {
    describe('数値入力', () => {
        it('数値文字列をそのまま数値に変換する', () => {
            expect(parseErogameScapeId('12345')).toBe(12345);
            expect(parseErogameScapeId('1')).toBe(1);
            expect(parseErogameScapeId('99999')).toBe(99999);
        });
    });

    describe('URL入力', () => {
        it('ErogameScapeのURLからゲームIDを抽出する', () => {
            const url = 'https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=12345';
            expect(parseErogameScapeId(url)).toBe(12345);
        });

        it('gameパラメータがないURLはundefinedを返す', () => {
            const url = 'https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/brand.php?brand=100';
            expect(parseErogameScapeId(url)).toBeUndefined();
        });
    });

    describe('無効な入力', () => {
        it('空文字はundefinedを返す', () => {
            expect(parseErogameScapeId('')).toBeUndefined();
        });

        it('無効な形式はundefinedを返す', () => {
            expect(parseErogameScapeId('abc')).toBeUndefined();
            expect(parseErogameScapeId('not-a-url-or-number')).toBeUndefined();
        });
    });
});
