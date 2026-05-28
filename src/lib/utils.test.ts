import { afterEach, beforeEach, describe, it, expect, vi } from 'vitest';
import {
    createLocalStorageCache,
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

describe('createLocalStorageCache', () => {
    beforeEach(() => {
        localStorage.clear();
        vi.useFakeTimers();
        vi.setSystemTime(new Date('2026-05-26T00:00:00Z'));
    });

    afterEach(() => {
        vi.useRealTimers();
        localStorage.clear();
    });

    it('同じバージョンの有効期限内キャッシュを使う', async () => {
        const fetcher = vi.fn(async () => 'fresh');
        const getter = createLocalStorageCache<'master', string>(
            'versioned-cache',
            fetcher,
            { version: 3, invalidateMilliseconds: 1000 * 60 * 60 * 24 * 7 }
        );

        await expect(getter('master')).resolves.toBe('fresh');
        await expect(getter('master')).resolves.toBe('fresh');

        expect(fetcher).toHaveBeenCalledTimes(1);
        expect(JSON.parse(localStorage.getItem('versioned-cache') ?? '{}')).toMatchObject({
            master: { value: 'fresh', version: 3 },
        });
    });

    it('バージョンが違うキャッシュは破棄する', async () => {
        localStorage.setItem(
            'versioned-cache',
            JSON.stringify({
                master: { value: 'old', createdAt: Date.now(), version: 2 },
            })
        );
        const fetcher = vi.fn(async () => 'new');
        const getter = createLocalStorageCache<'master', string>(
            'versioned-cache',
            fetcher,
            { version: 3, invalidateMilliseconds: 1000 * 60 * 60 * 24 * 7 }
        );

        await expect(getter('master')).resolves.toBe('new');

        expect(fetcher).toHaveBeenCalledTimes(1);
    });

    it('期限切れキャッシュは破棄する', async () => {
        const fetcher = vi
            .fn<[], Promise<string>>()
            .mockResolvedValueOnce('old')
            .mockResolvedValueOnce('new');
        const getter = createLocalStorageCache<'master', string>(
            'expiring-cache',
            fetcher,
            { version: 1, invalidateMilliseconds: 1000 * 60 * 60 * 24 * 7 }
        );

        await expect(getter('master')).resolves.toBe('old');
        vi.setSystemTime(new Date('2026-06-03T00:00:00Z'));
        await expect(getter('master')).resolves.toBe('new');

        expect(fetcher).toHaveBeenCalledTimes(2);
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

import { handleExternalLink, handleMarkdownClick } from './utils';
import { open as tauriOpen } from "@tauri-apps/plugin-shell";

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: vi.fn(),
}));

describe('handleExternalLink', () => {
  let originalWindowOpen: any;
  let originalTauriInternals: any;

  beforeEach(() => {
    originalWindowOpen = window.open;
    window.open = vi.fn();
    vi.mocked(tauriOpen).mockClear();
    
    // window.__TAURI_INTERNALS__ の状態を保存
    originalTauriInternals = (window as any).__TAURI_INTERNALS__;
  });

  afterEach(() => {
    window.open = originalWindowOpen;
    (window as any).__TAURI_INTERNALS__ = originalTauriInternals;
  });

  it('Tauri環境下であれば tauriOpen が呼び出されること', async () => {
    (window as any).__TAURI_INTERNALS__ = {};
    const url = 'https://google.com';
    await handleExternalLink(url);
    expect(tauriOpen).toHaveBeenCalledWith(url);
    expect(window.open).not.toHaveBeenCalled();
  });

  it('Web環境下であれば window.open が呼び出されること', async () => {
    (window as any).__TAURI_INTERNALS__ = undefined;
    const url = 'https://google.com';
    await handleExternalLink(url);
    expect(tauriOpen).not.toHaveBeenCalled();
    expect(window.open).toHaveBeenCalledWith(url, '_blank', 'noopener,noreferrer');
  });

  it('http/https 以外のURLは無視されること', async () => {
    const url = 'file:///C:/path/to/file';
    await handleExternalLink(url);
    expect(tauriOpen).not.toHaveBeenCalled();
    expect(window.open).not.toHaveBeenCalled();
  });
});

describe('handleMarkdownClick', () => {
  let originalWindowOpen: any;
  let originalTauriInternals: any;

  beforeEach(() => {
    originalWindowOpen = window.open;
    window.open = vi.fn();
    vi.mocked(tauriOpen).mockClear();
    (window as any).__TAURI_INTERNALS__ = undefined;
  });

  afterEach(() => {
    window.open = originalWindowOpen;
    (window as any).__TAURI_INTERNALS__ = originalTauriInternals;
  });

  it('a要素の外部リンククリック時にイベントが抑制され、window.open が呼ばれること', () => {
    const anchor = document.createElement('a');
    anchor.setAttribute('href', 'https://google.com');
    
    const event = {
      target: anchor,
      preventDefault: vi.fn(),
    } as unknown as MouseEvent;

    handleMarkdownClick(event);

    expect(event.preventDefault).toHaveBeenCalled();
    expect(window.open).toHaveBeenCalledWith('https://google.com', '_blank', 'noopener,noreferrer');
  });

  it('外部リンクではないa要素は抑制されず、処理されないこと', () => {
    const anchor = document.createElement('a');
    anchor.setAttribute('href', '#target');
    
    const event = {
      target: anchor,
      preventDefault: vi.fn(),
    } as unknown as MouseEvent;

    handleMarkdownClick(event);

    expect(event.preventDefault).not.toHaveBeenCalled();
    expect(window.open).not.toHaveBeenCalled();
  });
});
