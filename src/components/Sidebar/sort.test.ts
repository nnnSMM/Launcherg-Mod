import { describe, it, expect } from 'vitest';
import { sort } from './sort';
import type { CollectionElement } from '@/lib/types';

const createMockElement = (overrides: Partial<CollectionElement> = {}): CollectionElement => ({
    id: 1,
    gamename: 'テストゲーム',
    gamenameRuby: 'てすとげーむ',
    brandname: 'テストブランド',
    brandnameRuby: 'てすとぶらんど',
    sellday: '2024-01-01',
    isNukige: false,
    installAt: null,
    firstPlayAt: null,
    lastPlayAt: null,
    likeAt: null,
    playStatus: 0,
    totalPlayTimeSeconds: 0,
    registeredAt: '2024-01-01',
    exePath: '',
    lnkPath: '',
    icon: '',
    thumbnail: '',
    thumbnailWidth: null,
    thumbnailHeight: null,
    updatedAt: '2024-01-01',
    ...overrides,
});

describe('sort', () => {
    describe('gamename（タイトル）ソート', () => {
        it('gamename-ascで名前の昇順にソートされる', () => {
            const elements = [
                createMockElement({ id: 1, gamename: 'Cゲーム', gamenameRuby: 'しーげーむ' }),
                createMockElement({ id: 2, gamename: 'Aゲーム', gamenameRuby: 'えーげーむ' }),
                createMockElement({ id: 3, gamename: 'Bゲーム', gamenameRuby: 'びーげーむ' }),
            ];

            const result = sort(elements, 'gamename-asc');

            expect(result).toHaveLength(1);
            expect(result[0].label).toBe('すべて');
            expect(result[0].elements.map(e => e.gamename)).toEqual(['Aゲーム', 'Bゲーム', 'Cゲーム']);
        });

        it('gamename-descで名前の降順にソートされる', () => {
            const elements = [
                createMockElement({ id: 1, gamename: 'Aゲーム', gamenameRuby: 'えーげーむ' }),
                createMockElement({ id: 2, gamename: 'Cゲーム', gamenameRuby: 'しーげーむ' }),
                createMockElement({ id: 3, gamename: 'Bゲーム', gamenameRuby: 'びーげーむ' }),
            ];

            const result = sort(elements, 'gamename-desc');

            expect(result[0].elements.map(e => e.gamename)).toEqual(['Cゲーム', 'Bゲーム', 'Aゲーム']);
        });
    });

    describe('境界値テスト', () => {
        it('空配列を渡すとラベル付きの空配列を返す', () => {
            const result = sort([], 'gamename-asc');

            expect(result).toHaveLength(1);
            expect(result[0].elements).toHaveLength(0);
        });

        it('1要素の配列はそのまま返す', () => {
            const elements = [createMockElement({ gamename: '唯一のゲーム' })];

            const result = sort(elements, 'gamename-asc');

            expect(result[0].elements).toHaveLength(1);
            expect(result[0].elements[0].gamename).toBe('唯一のゲーム');
        });
    });

    describe('sellyear（発売年）ソート', () => {
        it('発売年ごとにグループ化される', () => {
            const elements = [
                createMockElement({ id: 1, gamename: 'ゲーム2024', sellday: '2024-06-15' }),
                createMockElement({ id: 2, gamename: 'ゲーム2023A', sellday: '2023-01-01' }),
                createMockElement({ id: 3, gamename: 'ゲーム2023B', sellday: '2023-12-31' }),
            ];

            const result = sort(elements, 'sellyear-desc');

            expect(result.length).toBeGreaterThanOrEqual(2);
            expect(result.map(g => g.label)).toContain('2024');
            expect(result.map(g => g.label)).toContain('2023');
        });
    });

    describe('日本語ソート', () => {
        it('日本語タイトルはふりがな順でソートされる', () => {
            const elements = [
                createMockElement({ gamename: 'わ行ゲーム', gamenameRuby: 'わぎょうげーむ' }),
                createMockElement({ gamename: 'あ行ゲーム', gamenameRuby: 'あぎょうげーむ' }),
                createMockElement({ gamename: 'か行ゲーム', gamenameRuby: 'かぎょうげーむ' }),
            ];

            const result = sort(elements, 'gamename-asc');
            const names = result[0].elements.map(e => e.gamename);

            expect(names).toEqual(['あ行ゲーム', 'か行ゲーム', 'わ行ゲーム']);
        });

        it('英語タイトルは日本語より先に来る', () => {
            const elements = [
                createMockElement({ gamename: '日本語ゲーム', gamenameRuby: 'にほんごげーむ' }),
                createMockElement({ gamename: 'English Game', gamenameRuby: 'english game' }),
            ];

            const result = sort(elements, 'gamename-asc');
            const names = result[0].elements.map(e => e.gamename);

            expect(names[0]).toBe('English Game');
        });
    });
});
