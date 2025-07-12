import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  // ...
  build: {
    // build dist into `src-tauri/` so that we can include it in Python sdist
    outDir: "src-tauri/frontend",
  },
}));
