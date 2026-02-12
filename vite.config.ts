import vue from '@vitejs/plugin-vue';
import path from 'node:path';
import type { ConfigEnv, UserConfig } from 'vite';
import { defineConfig } from 'vite';
import vuetify from 'vite-plugin-vuetify';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(
  async (_: ConfigEnv): Promise<UserConfig> => ({
    plugins: [
      vue(),
      vuetify({ autoImport: true }), // Vuetify 自动导入
    ],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
        vue: 'vue/dist/vue.esm-bundler.js',
      },
    },
    clearScreen: false,
    server: {
      port: 1420,
      strictPort: true,
      host: host || '127.0.0.1',
      hmr: host
        ? {
            protocol: 'ws',
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ['**/src-tauri/**'],
      },
    },
    build: {
      outDir: 'dist',
      emptyOutDir: true,
      rollupOptions: {
        input: {
          main: './index.html',
          // "pdf-viewer": "./pdf-viewer.html",
        },
      },
    },
  })
);
