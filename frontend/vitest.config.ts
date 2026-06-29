import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'happy-dom',
    setupFiles: ['./tests/setup.ts'],
    // Playwright specs live under tests/e2e/playwright and are run by Playwright
    // (npm run test:e2e), NOT Vitest. Exclude them so the unit runner ignores them.
    exclude: [
      'node_modules/**',
      'dist/**',
      'tests/e2e/playwright/**',
      '**/*.playwright.*',
    ],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      // Scope coverage to the modules our unit suite actually exercises. The
      // storage layer is unit-tested directly; infra modules (db/redis/index/
      // config/routes) require live services and are covered by the smoke test
      // and Playwright e2e instead. The React client is covered by e2e too.
      // Expand this list (and keep the 60% bar) as real unit tests are added.
      include: ['server/storage.ts'],
      exclude: [
        'node_modules/',
        'tests/',
        '**/*.d.ts',
        '**/*.config.*',
        '**/dist/**',
        '**/drizzle/**',
        'server/vite.ts',
      ],
      thresholds: {
        lines: 60,
        functions: 60,
        branches: 60,
        statements: 60,
      },
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './client/src'),
      '@shared': path.resolve(__dirname, './shared'),
      '@server': path.resolve(__dirname, './server'),
    },
  },
});