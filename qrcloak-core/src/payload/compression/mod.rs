use bytes::Bytes;
use thiserror::Error;

use crate::format::{CompletePayload, CompressionSpec};

#[derive(Debug, Clone, Default)]
pub enum Compression {
    #[default]
    NoCompression,
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
