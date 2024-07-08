// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig } from "@solidjs/start/config";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";
import mkcert from "vite-plugin-mkcert";

import path from "path";
import os from "os";

export default defineConfig({
	vite: {
		plugins: [wasm(), topLevelAwait(), mkcert()],
		server: {
			fs: {
				strict: false,
			},
		},
	},
	server: {
		preset: "static",
		https: {
			cert: path.join(os.homedir(), ".vite-plugin-mkcert", "cert.pem"),
			key: path.join(os.homedir(), ".vite-plugin-mkcert", "dev.pem"),
		},
	},
});
