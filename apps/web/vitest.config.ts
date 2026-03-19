import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    // Force browser-compatible Svelte builds (not SSR) in tests
    conditions: ['browser'],
  },
  test: {
    environment: 'jsdom',
  },
});
