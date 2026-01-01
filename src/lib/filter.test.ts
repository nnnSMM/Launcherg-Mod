import { describe, it, expect, vi } from 'vitest';
import { collectionElementsToOptions, type Option } from './filter';
import type { CollectionElement } from './types';

// wanakanaのモック
vi.mock('wanakana', () => ({
    toHiragana: (s: string) => `hiragana(${s})`,
    toRomaji: (s: string) => `romaji(${s})`,
}));

describe('collectionElementsToOptions', () => {
    const createMockElement = (overrides: Partial<CollectionElement> = {}): CollectionElement => ({
        id: 1,
        gamename: 'テストゲーム',
        gamenameRuby: 'てすとげーむ',
        brandname: 'テストブランド',
        brandnameRuby: 'てすとぶらんど',
        sellday: '2024-01-01',
        isNukige: false,
        installAt: null,
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

    it('空配列は空配列を返す', () => {
        const result = collectionElementsToOptions([]);
        expect(result).toEqual([]);
    });

    it('要素をOption形式に変換する', () => {
        const elements = [createMockElement({ id: 1, gamename: 'Game1' })];
        const result = collectionElementsToOptions(elements);

        expect(result).toHaveLength(1);
        expect(result[0].label).toBe('Game1');
        expect(result[0].value).toBe(1);
    });

    it('otherLabelsにルビ変換とブランド名を含む', () => {
        const elements = [createMockElement({
            gamenameRuby: 'gameruby',
            brandname: 'BrandName',
            brandnameRuby: 'brandruby',
        })];
        const result = collectionElementsToOptions(elements);

        expect(result[0].otherLabels).toContain('hiragana(gameruby)');
        expect(result[0].otherLabels).toContain('romaji(gameruby)');
        expect(result[0].otherLabels).toContain('BrandName');
        expect(result[0].otherLabels).toContain('hiragana(brandruby)');
        expect(result[0].otherLabels).toContain('romaji(brandruby)');
    });

    it('複数要素を正しく変換する', () => {
        const elements = [
            createMockElement({ id: 1, gamename: 'Game1' }),
            createMockElement({ id: 2, gamename: 'Game2' }),
            createMockElement({ id: 3, gamename: 'Game3' }),
        ];
        const result = collectionElementsToOptions(elements);

        expect(result).toHaveLength(3);
        expect(result.map(o => o.value)).toEqual([1, 2, 3]);
    });
});
