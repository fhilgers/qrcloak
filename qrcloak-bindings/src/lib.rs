use bytes::Bytes;
use compression::{Compression, Decompression};
use encryption::{Decryption, Encryption};
use qrcloak_core::{
    format::CompletePayload,
    payload::{DecodingOpts, EncodingOpts, MergeResult, UnmergedPayloads},
};

extern crate alloc;

use wasm_bindgen::prelude::*;

mod as_js_value;
mod compression;
mod encryption;
mod payloads;

use payloads::Payloads;

#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct PayloadGenerator(qrcloak_core::payload::PayloadGenerator);

#[wasm_bindgen]
impl PayloadGenerator {
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

    pub fn generate(&self, data: &str) -> Result<CompletePayload, JsError> {
        Ok(self.0.generate(Bytes::copy_from_slice(data.as_bytes()))?)
    }
}

#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct PayloadSplitter(qrcloak_core::payload::PayloadSplitter);

#[wasm_bindgen]
impl PayloadSplitter {
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

#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct PayloadExtractor(qrcloak_core::payload::PayloadExtractor);

#[wasm_bindgen]
impl PayloadExtractor {
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

    pub fn extract(&self, payload: CompletePayload) -> Result<Vec<u8>, JsError> {
        Ok(self.0.extract(payload)?.into())
    }
}

#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct PayloadMerger(qrcloak_core::payload::PayloadMerger);

#[wasm_bindgen]
impl PayloadMerger {
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

#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct PayloadEncoder(qrcloak_core::payload::Encoder);

#[wasm_bindgen]
impl PayloadEncoder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_encoding(&self, encoding: EncodingOpts) -> Self {
        Self(self.0.with_encoding(encoding))
    }

    pub fn encode(&self, payloads: Payloads) -> Result<Vec<String>, JsError> {
        Ok(self.0.encode(payloads)?)
    }
}

#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct PayloadDecoder(qrcloak_core::payload::Decoder);

#[wasm_bindgen]
impl PayloadDecoder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_decoding(&self, opts: DecodingOpts) -> Self {
        Self(self.0.with_opts(opts))
    }

    pub fn decode(&self, payloads: &str) -> Result<Payloads, JsError> {
        Ok(self.0.decode(payloads.as_bytes())?.into())
    }
}
