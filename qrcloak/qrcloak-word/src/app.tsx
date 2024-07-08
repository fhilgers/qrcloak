// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { clientOnly } from "@solidjs/start";

const Inner = clientOnly(() => import("~/app-inner"));

export default function App() {
	return <Inner />;
}
