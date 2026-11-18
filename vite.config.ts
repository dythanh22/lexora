import { resolve } from 'node:path';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

// Hai cửa sổ, hai điểm vào: popup (index.html) và settings (settings.html).
// Settings chỉ được tạo khi cần (thiết kế §8.2).
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    target: 'es2022',
    rollupOptions: {
      input: {
        popup: resolve(__dirname, 'index.html'),
        settings: resolve(__dirname, 'settings.html')
      }
    }
  },
  test: { environment: 'node', include: ['src/**/*.test.ts'] }
});
