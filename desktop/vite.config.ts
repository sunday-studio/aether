import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
	plugins: [react(), tailwindcss(), tsconfigPaths()],

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	// 1. prevent Vite from obscuring rust errors
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: "ws",
					host,
					port: 1421,
				}
			: {
					overlay: true,
				},
		watch: {
			ignored: ["**/src-tauri/**"],
		},
	},
	// Prevent Tauri APIs from being optimized/bundled which causes HMR issues
	optimizeDeps: {
		exclude: ["@tauri-apps/api", "@tauri-apps/plugin-updater"],
	},
	// Ensure modules are properly rebuilt on changes
	build: {
		target: "esnext",
	},
}));
