use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use tsify_next::Tsify;

use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

use crate::serde_impl;
use crate::wrapper_impl;

#[derive(TryFromJsValue, Clone, Debug)]
#[wasm_bindgen]
pub struct GzipCompression(qrcloak_core::payload::GzipCompression);

#[wasm_bindgen]
impl GzipCompression {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(qrcloak_core::payload::GzipCompression)
    }
}

serde_impl!(GzipCompression);
wrapper_impl!(GzipCompression, qrcloak_core::payload::GzipCompression);

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Compression {
    NoCompression,
    Gzip(GzipCompression),
}

impl From<Compression> for qrcloak_core::payload::Compression {
    fn from(compression: Compression) -> Self {
        match compression {
            Compression::NoCompression => qrcloak_core::payload::Compression::NoCompression,
            Compression::Gzip(gzip) => qrcloak_core::payload::Compression::Gzip(gzip.into()),
        }
    }
}

impl Into<Compression> for qrcloak_core::payload::Compression {
    fn into(self) -> Compression {
        match self {
            qrcloak_core::payload::Compression::NoCompression => Compression::NoCompression,
            qrcloak_core::payload::Compression::Gzip(gzip) => Compression::Gzip(gzip.into()),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Decompression {
    NoCompression,
    Gzip(GzipCompression),
}

impl From<Decompression> for qrcloak_core::payload::Decompression {
    fn from(value: Decompression) -> Self {
        match value {
            Decompression::NoCompression => qrcloak_core::payload::Decompression::NoCompression,
            Decompression::Gzip(gzip) => qrcloak_core::payload::Decompression::Gzip(gzip.into()),
        }
    }
}

impl Into<Decompression> for qrcloak_core::payload::Decompression {
    fn into(self) -> Decompression {
        match self {
            qrcloak_core::payload::Decompression::NoCompression => Decompression::NoCompression,
            qrcloak_core::payload::Decompression::Gzip(gzip) => Decompression::Gzip(gzip.into()),
        }
    }
}
