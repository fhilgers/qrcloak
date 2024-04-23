import { defineConfig } from "@solidjs/start/config";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  vite: {
    plugins: [wasm(), topLevelAwait()],
  },
  server: {
    preset: "static",
  },
});
