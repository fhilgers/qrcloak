use bytes::Bytes;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

use super::CompletePayload;

#[derive(Debug, Clone, Default)]
pub enum Compression {
    #[default]
    NoCompression,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, Default)]
pub enum CompressionSpec {
    #[default]
    NoCompression,
}

impl CompressionSpec {
    pub fn no_compression(&self) -> bool {
        match self {
            CompressionSpec::NoCompression => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Decompression {
    #[default]
    NoCompression,
}

#[derive(Debug, Error)]
pub enum CompressionError {}

impl Compression {
    pub fn process(&self, data: Bytes) -> Result<Bytes, CompressionError> {
        match self {
            Compression::NoCompression => Ok(data),
        }
    }

    pub fn spec(&self) -> CompressionSpec {
        match self {
            Compression::NoCompression => CompressionSpec::NoCompression,
        }
    }
}

#[derive(Debug, Error)]
pub enum DecompressionError {
    #[error("tried to decompress with {tried:?} but payload is compressed with {payload:?}")]
    SpecMismtach {
        payload: CompressionSpec,
        tried: Decompression,
    },
}

impl Decompression {
    pub fn process(&self, data: &mut CompletePayload) -> Result<(), DecompressionError> {
        match (self, &data.compression) {
            (Decompression::NoCompression, CompressionSpec::NoCompression) => Ok(()),
        }
    }
}
