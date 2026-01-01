import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
    plugins: [svelte({ hot: !process.env.VITEST }) as any],
    test: {
        globals: true,
        environment: 'jsdom',
        include: ['src/**/*.{test,spec}.{js,ts}'],
    },
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src'),
        },
    },
});
