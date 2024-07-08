// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import {
	AgeIdentity,
	PayloadDecoder,
	PayloadEncoder,
	PayloadExtractor,
	PayloadGenerator,
	PayloadMerger,
	PayloadSplitter,
} from "@fhilgers/qrcloak";

import { expect, test } from "bun:test";

test("roundtrip", () => {
	let data = "hello from typescript";

	let identity = AgeIdentity.generate();
	let recipient = identity.to_public();

	let payload = new PayloadGenerator()
		.with_encryption({ AgeKey: { recipients: [recipient] } })
		.generate(data);

	let splits = new PayloadSplitter().with_splits(4).split(payload);

	expect(splits.length).toBe(4);

	let encoded = new PayloadEncoder()
		.with_encoding({ Json: { pretty: true, merge: true } })
		.encode(splits);

	expect(encoded.length).toBe(1);

	let decoded = new PayloadDecoder().with_decoding("Json").decode(encoded[0]);

	let merged = new PayloadMerger().merge(decoded);

	expect(merged.complete.length).toBe(1);
	expect(merged.incomplete.misconfigured.length).toBe(0);
	expect(merged.incomplete.partials.size).toBe(0);

	let extracted = new PayloadExtractor()
		.with_decryption({ AgeKey: { identities: [identity] } })
		.extract(merged.complete[0]);

	let str = new TextDecoder().decode(extracted);

	expect(str).toBe(data);
});
