import { describe, it, expect } from 'vitest';
import {
    formatPlayTime,
    formatLastPlayed,
    isNotNullOrUndefined,
    rand,
} from './utils';

describe('formatPlayTime', () => {
    it('2時間未満は分で表示する', () => {
        expect(formatPlayTime(0)).toBe('0分');
        expect(formatPlayTime(60)).toBe('1分');
        expect(formatPlayTime(3599)).toBe('59分');
        expect(formatPlayTime(7199)).toBe('119分');
    });

    it('2時間以上は時間で表示する', () => {
        expect(formatPlayTime(7200)).toBe('2時間');
        expect(formatPlayTime(9000)).toBe('2.5時間');
        expect(formatPlayTime(36000)).toBe('10時間');
    });
});

describe('formatLastPlayed', () => {
    it('null/undefinedは空文字を返す', () => {
        expect(formatLastPlayed(null)).toBe('');
        expect(formatLastPlayed(undefined)).toBe('');
    });

    it('今日の日付は「今日」を返す', () => {
        const today = new Date().toISOString();
        expect(formatLastPlayed(today)).toBe('今日');
    });

    it('昨日の日付は「昨日」を返す', () => {
        const yesterday = new Date();
        yesterday.setDate(yesterday.getDate() - 1);
        expect(formatLastPlayed(yesterday.toISOString())).toBe('昨日');
    });

    it('2〜14日前は「X日前」を返す', () => {
        const twoDaysAgo = new Date();
        twoDaysAgo.setDate(twoDaysAgo.getDate() - 2);
        expect(formatLastPlayed(twoDaysAgo.toISOString())).toBe('2日前');

        const fourteenDaysAgo = new Date();
        fourteenDaysAgo.setDate(fourteenDaysAgo.getDate() - 14);
        expect(formatLastPlayed(fourteenDaysAgo.toISOString())).toBe('14日前');
    });

    it('15日以上前は日付を表示する', () => {
        const oldDate = new Date('2024-01-15');
        const result = formatLastPlayed(oldDate.toISOString());
        expect(result).toMatch(/\d{4}\/\d{1,2}\/\d{1,2}/);
    });
});

describe('isNotNullOrUndefined', () => {
    it('nullとundefinedにはfalseを返す', () => {
        expect(isNotNullOrUndefined(null)).toBe(false);
        expect(isNotNullOrUndefined(undefined)).toBe(false);
    });

    it('有効な値にはtrueを返す', () => {
        expect(isNotNullOrUndefined(0)).toBe(true);
        expect(isNotNullOrUndefined('')).toBe(true);
        expect(isNotNullOrUndefined(false)).toBe(true);
        expect(isNotNullOrUndefined([])).toBe(true);
        expect(isNotNullOrUndefined({})).toBe(true);
    });
});

describe('rand', () => {
    it('0以上max未満の範囲で値を返す', () => {
        for (let i = 0; i < 100; i++) {
            const value = rand(100);
            expect(value).toBeGreaterThanOrEqual(0);
            expect(value).toBeLessThan(100);
        }
    });

    it('デフォルトは0〜100000未満', () => {
        const value = rand();
        expect(value).toBeGreaterThanOrEqual(0);
        expect(value).toBeLessThan(100000);
    });
});

import { convertSpecialCharacters } from './utils';

describe('convertSpecialCharacters', () => {
    it('HTMLエンティティをデコードする', () => {
        expect(convertSpecialCharacters('&amp;')).toBe('&');
        expect(convertSpecialCharacters('&lt;')).toBe('<');
        expect(convertSpecialCharacters('&gt;')).toBe('>');
        expect(convertSpecialCharacters('&quot;')).toBe('"');
    });

    it('通常の文字列はそのまま返す', () => {
        expect(convertSpecialCharacters('Hello World')).toBe('Hello World');
    });

    it('空文字列はそのまま返す', () => {
        expect(convertSpecialCharacters('')).toBe('');
    });
});
