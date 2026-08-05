import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

declare const process: {
  env: Record<string, string | undefined>;
};

// Tauri sets this for physical iOS and Android devices so the WebView can reach Vite.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: host || false,
    port: 1421,
    strictPort: true,
    hmr: host
      ? {
          host,
          port: 1422,
          protocol: "ws"
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"]
    }
  },
  optimizeDeps: {
    exclude: ["@tauri-apps/api"]
  },
  build: {
    target: "esnext"
  }
});
