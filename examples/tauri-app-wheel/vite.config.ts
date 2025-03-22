import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  build: {
    // build dist into `python/src/tauri_app_wheel/` so that we can include it in python sdist and wheel
    outDir: "python/src/tauri_app_wheel/frontend",
  },
  // we set fixed port in python code, so it's better to fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**", "**/.venv/**"],
    },
  },
}));
