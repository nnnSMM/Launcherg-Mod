import { describe, it, expect } from 'vitest';
import { FILTER_BY_ATTRIBUTE } from './searchAttributes';
import type { CollectionElement } from '@/lib/types';
import { PlayStatus } from '@/lib/types';

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
    playStatus: PlayStatus.Unplayed,
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

describe('FILTER_BY_ATTRIBUTE', () => {
    describe('nukige', () => {
        it('isNukigeがtrueの要素のみを返す', () => {
            const elements = [
                createMockElement({ id: 1, isNukige: true }),
                createMockElement({ id: 2, isNukige: false }),
                createMockElement({ id: 3, isNukige: true }),
            ];

            const result = FILTER_BY_ATTRIBUTE.nukige(elements);

            expect(result).toHaveLength(2);
            expect(result.map(e => e.id)).toEqual([1, 3]);
        });
    });

    describe('not_nukige', () => {
        it('isNukigeがfalseの要素のみを返す', () => {
            const elements = [
                createMockElement({ id: 1, isNukige: true }),
                createMockElement({ id: 2, isNukige: false }),
            ];

            const result = FILTER_BY_ATTRIBUTE.not_nukige(elements);

            expect(result).toHaveLength(1);
            expect(result[0].id).toBe(2);
        });
    });

    describe('exist_path', () => {
        it('installAtがnullでない要素のみを返す', () => {
            const elements = [
                createMockElement({ id: 1, installAt: '2024-01-01' }),
                createMockElement({ id: 2, installAt: null }),
            ];

            const result = FILTER_BY_ATTRIBUTE.exist_path(elements);

            expect(result).toHaveLength(1);
            expect(result[0].id).toBe(1);
        });
    });

    describe('like', () => {
        it('likeAtがnullでない要素のみを返す', () => {
            const elements = [
                createMockElement({ id: 1, likeAt: '2024-01-01' }),
                createMockElement({ id: 2, likeAt: null }),
            ];

            const result = FILTER_BY_ATTRIBUTE.like(elements);

            expect(result).toHaveLength(1);
            expect(result[0].id).toBe(1);
        });
    });

    describe('playStatus', () => {
        it('unplayedはplayStatus=0の要素のみを返す', () => {
            const elements = [
                createMockElement({ id: 1, playStatus: PlayStatus.Unplayed }),
                createMockElement({ id: 2, playStatus: PlayStatus.Playing }),
                createMockElement({ id: 3, playStatus: PlayStatus.Cleared }),
            ];

            const result = FILTER_BY_ATTRIBUTE.unplayed(elements);

            expect(result).toHaveLength(1);
            expect(result[0].id).toBe(1);
        });

        it('playingはplayStatus=1の要素のみを返す', () => {
            const elements = [
                createMockElement({ id: 1, playStatus: PlayStatus.Playing }),
                createMockElement({ id: 2, playStatus: PlayStatus.Cleared }),
            ];

            const result = FILTER_BY_ATTRIBUTE.playing(elements);

            expect(result).toHaveLength(1);
            expect(result[0].id).toBe(1);
        });

        it('clearedはplayStatus=2の要素のみを返す', () => {
            const elements = [
                createMockElement({ id: 1, playStatus: PlayStatus.Cleared }),
                createMockElement({ id: 2, playStatus: PlayStatus.Unplayed }),
            ];

            const result = FILTER_BY_ATTRIBUTE.cleared(elements);

            expect(result).toHaveLength(1);
            expect(result[0].id).toBe(1);
        });
    });

    describe('空配列', () => {
        it('空配列を渡すと空配列を返す', () => {
            expect(FILTER_BY_ATTRIBUTE.nukige([])).toEqual([]);
            expect(FILTER_BY_ATTRIBUTE.unplayed([])).toEqual([]);
            expect(FILTER_BY_ATTRIBUTE.like([])).toEqual([]);
        });
    });
});
