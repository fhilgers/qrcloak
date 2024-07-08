// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use bytes::Bytes;
use compression::{Compression, Decompression};
use encryption::{Decryption, Encryption};
use qrcloak_core::{
    format::CompletePayload,
    payload::{DecodingOpts, EncodingOpts, MergeResult, UnmergedPayloads},
};

extern crate alloc;

use uniffi::{Error, Object};
use wasm_bindgen::prelude::*;

mod as_js_value;
mod compression;
mod encryption;
mod payloads;
mod uniffi_object_clone;

use payloads::Payloads;

uniffi::setup_scaffolding!();

#[derive(Default, Clone, Object)]
#[wasm_bindgen]
pub struct PayloadGenerator(qrcloak_core::payload::PayloadGenerator);

#[uniffi::export]
#[wasm_bindgen]
impl PayloadGenerator {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_encryption(&self, encryption: Encryption) -> Self {
        Self(self.0.clone().with_encryption(encryption.into()))
    }

    pub fn with_compression(&self, compression: Compression) -> Self {
        Self(self.0.clone().with_compression(compression.into()))
    }

    pub fn generate(&self, data: &str) -> Result<CompletePayload, GenericError> {
        Ok(self.0.generate(Bytes::copy_from_slice(data.as_bytes()))?)
    }
}

#[derive(Default, Clone, Object)]
#[wasm_bindgen]
pub struct PayloadSplitter(qrcloak_core::payload::PayloadSplitter);

#[uniffi::export]
#[wasm_bindgen]
impl PayloadSplitter {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_splits(&self, splits: u32) -> Self {
        Self(self.0.clone().with_splits(splits))
    }

    pub fn split(&self, payload: CompletePayload) -> Payloads {
        self.0.split(payload).collect()
    }
}

#[derive(Default, Clone, Object)]
#[wasm_bindgen]
pub struct PayloadExtractor(qrcloak_core::payload::PayloadExtractor);

#[uniffi::export]
#[wasm_bindgen]
impl PayloadExtractor {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_decryption(&self, decryption: Decryption) -> Self {
        Self(self.0.clone().with_decryption(decryption.into()))
    }

    pub fn with_decompression(&self, decompression: Decompression) -> Self {
        Self(self.0.clone().with_decompression(decompression.into()))
    }

    pub fn extract(&self, payload: CompletePayload) -> Result<Vec<u8>, GenericError> {
        Ok(self.0.extract(payload)?.into())
    }
}

#[derive(Default, Clone, Object)]
#[wasm_bindgen]
pub struct PayloadMerger(qrcloak_core::payload::PayloadMerger);

#[uniffi::export]
#[wasm_bindgen]
impl PayloadMerger {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_unmerged(&self, unmerged: UnmergedPayloads) -> Self {
        Self(self.0.clone().with_unmerged(unmerged))
    }

    pub fn merge(&self, payloads: Payloads) -> MergeResult {
        self.0.clone().merge(payloads).into()
    }
}

#[derive(Default, Clone, Object)]
#[wasm_bindgen]
pub struct PayloadEncoder(qrcloak_core::payload::Encoder);

#[uniffi::export]
#[wasm_bindgen]
impl PayloadEncoder {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_encoding(&self, encoding: EncodingOpts) -> Self {
        Self(self.0.with_encoding(encoding))
    }

    pub fn encode(&self, payloads: Payloads) -> Result<Vec<String>, GenericError> {
        Ok(self.0.encode(payloads)?)
    }
}

#[derive(Default, Clone, Object)]
#[wasm_bindgen]
pub struct PayloadDecoder(qrcloak_core::payload::Decoder);

#[uniffi::export]
#[wasm_bindgen]
impl PayloadDecoder {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_decoding(&self, opts: DecodingOpts) -> Self {
        Self(self.0.with_opts(opts))
    }

    pub fn decode(&self, payloads: &str) -> Result<Payloads, GenericError> {
        Ok(self.0.decode(payloads.as_bytes())?.into())
    }
}

// TODO: this is necessary because of https://github.com/mozilla/uniffi-rs/issues/1605
#[derive(Debug, thiserror::Error, Error)]
#[uniffi(flat_error)]
pub enum GenericError {
    #[error(transparent)]
    DecodingError(#[from] qrcloak_core::payload::DecodingError),
    #[error(transparent)]
    EncodingError(#[from] qrcloak_core::payload::EncodingError),
    #[error(transparent)]
    ExtractionError(#[from] qrcloak_core::payload::PayloadExtractionError),
    #[error(transparent)]
    GenerationError(#[from] qrcloak_core::payload::PayloadGenerationError),
}

impl Into<JsValue> for GenericError {
    fn into(self) -> JsValue {
        JsValue::from(self.to_string())
    }
}
