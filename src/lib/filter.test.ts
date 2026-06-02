import { afterEach, beforeEach, describe, it, expect, vi } from 'vitest';
import { collectionElementsToOptions, useFilter, type Option } from './filter';
import type { CollectionElement } from './types';
import { get, writable, type Writable } from 'svelte/store';

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

describe('useFilter', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('options更新時にquery購読を増やさず、最新候補で絞り込む', () => {
        let querySubscribeCount = 0;
        let queryUnsubscribeCount = 0;
        let latestQuery = "";
        let querySubscriber: (value: string) => void = () => {};
        const queryStore = {
            subscribe(run: (value: string) => void) {
                querySubscribeCount += 1;
                querySubscriber = run;
                run(latestQuery);
                return () => {
                    queryUnsubscribeCount += 1;
                };
            },
            set(value: string) {
                latestQuery = value;
                querySubscriber(value);
            },
            update(updater: (value: string) => string) {
                latestQuery = updater(latestQuery);
                querySubscriber(latestQuery);
            },
        } as Writable<string>;

        let options: Option<number>[] = [
            { label: 'Alpha', value: 1 },
            { label: 'Beta', value: 2 },
        ];
        const optionsStore = writable(options);
        const { filtered } = useFilter(queryStore, optionsStore, () => options);
        const unsubscribe = filtered.subscribe(() => {});

        expect(querySubscribeCount).toBe(1);

        queryStore.set('alp');
        vi.advanceTimersByTime(200);
        expect(get(filtered).map((option) => option.value)).toEqual([1]);

        options = [
            { label: 'Alpha', value: 1 },
            { label: 'Alpine', value: 3 },
            { label: 'Beta', value: 2 },
        ];
        optionsStore.set(options);

        expect(querySubscribeCount).toBe(1);
        expect(get(filtered).map((option) => option.value)).toEqual([1, 3]);

        unsubscribe();
        expect(queryUnsubscribeCount).toBe(1);
    });
});
