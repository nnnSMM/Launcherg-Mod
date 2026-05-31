import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import sveltePreprocess from 'svelte-preprocess';
import path from 'path';

export default defineConfig({
    plugins: [
        svelte({
            hot: !process.env.VITEST,
            preprocess: [
                sveltePreprocess({
                    typescript: true,
                }),
            ],
        }) as any
    ],
    test: {
        globals: true,
        environment: 'jsdom',
        include: ['src/**/*.{test,spec}.{js,ts}'],
        // ネットワークアクセスが必要なデータ更新スクリプトは通常のテスト実行から除外
        exclude: ['src/lib/fetch-demo-descriptions.test.ts'],
    },
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src'),
        },
    },
});
