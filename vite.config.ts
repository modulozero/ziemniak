import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    write: true,
  },
  resolve: {
    alias: {
      "@app": "./src/",
    },
  },
});
