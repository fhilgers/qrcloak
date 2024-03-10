import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import devCerts from "office-addin-dev-certs";
import basicSsl from "@vitejs/plugin-basic-ssl";

import devtools from "solid-devtools/vite";

export default defineConfig(async ({ command, mode }) => {
  return {
    plugins: [devtools(), solidPlugin(), wasm(), topLevelAwait()],
    server: {
      port: 3000,
      https: await devCerts.getHttpsServerOptions(),
    },
    build: {
      target: "esnext",
    },
  };
});
