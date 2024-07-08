// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import {
	Encryption as QrCloakEncryption,
	Compression as QrCloakCompression,
	PayloadGenerator,
	PayloadSplitter,
	PayloadEncoder,
	Payloads,
	Passphrase,
	AgeRecipient,
	GzipCompression,
} from "@fhilgers/qrcloak";

import { toDataURL } from "qrcode";

export const EncryptionOptions = [
	"No Encryption",
	"Age Passphrase",
	"Age Recipient",
] as const;
export type Encryption = (typeof EncryptionOptions)[number];
export const encryptionOptions = EncryptionOptions.map((e) => e);

export const CompressionOptions = ["No Compression", "Gzip"] as const;
export type Compression = (typeof CompressionOptions)[number];
export const compressionOptions = CompressionOptions.map((e) => e);

function encryptionToQrCloakEncryption(
	encryption: Encryption,
	secret: string,
): QrCloakEncryption {
	switch (encryption) {
		case "No Encryption":
			return "NoEncryption";
		case "Age Passphrase":
			return { AgePassphrase: { passphrase: new Passphrase(secret) } };
		case "Age Recipient":
			return { AgeKey: { recipients: [new AgeRecipient(secret)] } };
	}
}

function compressionToQrCloakCompression(
	compression: Compression,
): QrCloakCompression {
	switch (compression) {
		case "No Compression":
			return "NoCompression";
		case "Gzip":
			return { Gzip: { gzip: new GzipCompression() } };
	}
}

export function createPayloads(
	text: string,
	secret: string,
	splits: number,
	encryption: Encryption,
	compression: Compression,
): Payloads {
	const qrCloakEncryptoin = encryptionToQrCloakEncryption(encryption, secret);
	const qrCloakCompression = compressionToQrCloakCompression(compression);

	const payload = new PayloadGenerator()
		.with_encryption(qrCloakEncryptoin)
		.with_compression(qrCloakCompression)
		.generate(text);

	if (splits == 0 || splits > text.length) {
		return [payload];
	}

	return new PayloadSplitter().with_splits(splits).split(payload);
}

export function encodePayloads(payloads: Payloads): string[] {
	const encoded = new PayloadEncoder()
		.with_encoding({ Json: { pretty: false, merge: false } })
		.encode(payloads);

	return encoded;
}

export async function toSvg(encoded: string): Promise<string> {
	return await new Promise((resolve, reject) => {
		toDataURL(encoded, function (err, url) {
			if (err) {
				reject(err);
			}
			resolve(url);
		});
	});
}
