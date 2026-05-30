import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

import {
    commandGetAllGameCacheLastUpdated,
    commandGetAppSetting,
    commandGetCollectionElementDailyPlayTimes,
    commandPlayGame,
} from './command';

describe('command.ts', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.resetAllMocks();
    });

    describe('commandGetAppSetting', () => {
        it('invokeの結果をそのまま返す', async () => {
            (invoke as ReturnType<typeof vi.fn>).mockResolvedValue('dark');

            const result = await commandGetAppSetting('theme');

            expect(result).toBe('dark');
        });

        it('invokeが正しい引数で呼ばれることを検証', async () => {
            (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(null);

            await commandGetAppSetting('language');

            expect(invoke).toHaveBeenCalledWith('get_app_setting', { key: 'language' });
            expect(invoke).toHaveBeenCalledTimes(1);
        });
    });

    describe('commandGetAllGameCacheLastUpdated', () => {
        it('invokeの結果を変換して返す', async () => {
            const mockDate = '2024-06-15T12:00:00Z';
            (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([123, mockDate]);

            const result = await commandGetAllGameCacheLastUpdated();

            expect(result.id).toBe(123);
            expect(result.date).toBeInstanceOf(Date);
            expect(result.date.getTime()).toBe(new Date(mockDate).getTime());
        });
    });

    describe('エラーハンドリング', () => {
        it('invokeがエラーを投げた場合、そのエラーが伝播する', async () => {
            (invoke as ReturnType<typeof vi.fn>).mockRejectedValue(
                new Error('Network error')
            );

            await expect(commandGetAppSetting('test')).rejects.toThrow('Network error');
        });
    });

    describe('commandGetCollectionElementDailyPlayTimes', () => {
        it('日別プレイ時間取得コマンドを正しい引数で呼ぶ', async () => {
            const rows = [
                {
                    collectionElementId: 10,
                    playDate: '2026-05-30',
                    playTimeSeconds: 3600,
                },
            ];
            (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(rows);

            const result = await commandGetCollectionElementDailyPlayTimes(10);

            expect(result).toEqual(rows);
            expect(invoke).toHaveBeenCalledWith(
                'get_collection_element_daily_play_times',
                { collectionElementId: 10 },
            );
        });
    });

    describe('commandPlayGame', () => {
        it('引数によって異なる結果を返す', async () => {
            (invoke as ReturnType<typeof vi.fn>).mockImplementation(
                (cmd: string, args: { elementId: number }) => {
                    if (args.elementId === 1) {
                        return Promise.resolve(12345);
                    }
                    return Promise.resolve(null);
                }
            );

            expect(await commandPlayGame(1, false)).toBe(12345);
            expect(await commandPlayGame(999, false)).toBeNull();
        });
    });
});
