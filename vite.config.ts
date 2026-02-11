import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify from "vite-plugin-vuetify";
import type { UserConfig, ConfigEnv } from "vite";
import path from "node:path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(
  async (_: ConfigEnv): Promise<UserConfig> => ({
    plugins: [
      vue(),
      vuetify({ autoImport: true }), // Vuetify 自动导入
    ],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
    clearScreen: false,
    server: {
      port: 1420,
      strictPort: true,
      host: host || "127.0.0.1",
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ["**/src-tauri/**"],
      },
    },
    build: {
      outDir: "dist",
      emptyOutDir: true,
      rollupOptions: {
        input: {
          main: "./index.html",
          "pdf-viewer": "./pdf-viewer.html",
        },
      },
    },
  }),
);
