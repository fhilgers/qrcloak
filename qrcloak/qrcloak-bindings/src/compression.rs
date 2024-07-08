// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use tsify_next::Tsify;

use uniffi::Enum;
use uniffi::Object;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

use crate::serde_impl;
use crate::uniffi_object_clone;
use crate::wrapper_impl;

#[derive(TryFromJsValue, Clone, Debug, Object)]
#[wasm_bindgen]
pub struct GzipCompression(qrcloak_core::payload::GzipCompression);

#[uniffi::export]
#[wasm_bindgen]
impl GzipCompression {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(qrcloak_core::payload::GzipCompression)
    }
}

serde_impl!(GzipCompression);
wrapper_impl!(GzipCompression, qrcloak_core::payload::GzipCompression);
uniffi_object_clone!(GzipCompression);

#[derive(Tsify, Serialize, Deserialize, Enum)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Compression {
    NoCompression,
    Gzip { gzip: GzipCompression },
}

impl From<Compression> for qrcloak_core::payload::Compression {
    fn from(compression: Compression) -> Self {
        match compression {
            Compression::NoCompression => qrcloak_core::payload::Compression::NoCompression,
            Compression::Gzip { gzip } => qrcloak_core::payload::Compression::Gzip(gzip.into()),
        }
    }
}

impl Into<Compression> for qrcloak_core::payload::Compression {
    fn into(self) -> Compression {
        match self {
            qrcloak_core::payload::Compression::NoCompression => Compression::NoCompression,
            qrcloak_core::payload::Compression::Gzip(gzip) => {
                Compression::Gzip { gzip: gzip.into() }
            }
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Enum)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Decompression {
    NoCompression,
    Gzip { gzip: GzipCompression },
}

impl From<Decompression> for qrcloak_core::payload::Decompression {
    fn from(value: Decompression) -> Self {
        match value {
            Decompression::NoCompression => qrcloak_core::payload::Decompression::NoCompression,
            Decompression::Gzip { gzip } => qrcloak_core::payload::Decompression::Gzip(gzip.into()),
        }
    }
}

impl Into<Decompression> for qrcloak_core::payload::Decompression {
    fn into(self) -> Decompression {
        match self {
            qrcloak_core::payload::Decompression::NoCompression => Decompression::NoCompression,
            qrcloak_core::payload::Decompression::Gzip(gzip) => {
                Decompression::Gzip { gzip: gzip.into() }
            }
        }
    }
}
