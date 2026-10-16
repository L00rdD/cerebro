import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri expects a fixed port and fails if it is not available.
export default defineConfig({
  root: "ui",
  // Sprites and manifests are served from the repo-level `assets/` folder.
  publicDir: "../assets",
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**", "**/target/**"] } },
  build: { outDir: "../dist", emptyOutDir: true, target: "safari15" },
  test: { root: ".", include: ["ui/src/**/*.test.ts"] },
});
