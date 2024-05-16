use bytes::Bytes;
use thiserror::Error;

mod gzip;

use crate::format::{CompletePayload, CompressionSpec};

pub use gzip::GzipCompression;

#[derive(Debug, Clone, Default)]
pub enum Compression {
    #[default]
    NoCompression,
    Gzip(gzip::GzipCompression),
}

#[derive(Debug, Clone, Default)]
pub enum Decompression {
    #[default]
    NoCompression,
    Gzip(gzip::GzipCompression),
}

impl From<&Compression> for CompressionSpec {
    fn from(compression: &Compression) -> Self {
        match compression {
            Compression::NoCompression => CompressionSpec::NoCompression,
            Compression::Gzip(_) => CompressionSpec::Gzip,
        }
    }
}

#[derive(Debug, Error)]
pub enum CompressionError {}

impl Compression {
    pub fn process(&self, data: Bytes) -> Result<Bytes, CompressionError> {
        match self {
            Compression::NoCompression => Ok(data),
            Compression::Gzip(compression) => Ok(compression.compress(data)),
        }
    }

    pub fn spec(&self) -> CompressionSpec {
        self.into()
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
            (Decompression::Gzip(compression), CompressionSpec::Gzip) => {
                data.data = compression.decompress(data.data.clone());
                Ok(())
            }
            (tried, payload) => Err(DecompressionError::SpecMismtach {
                payload: payload.clone(),
                tried: tried.clone(),
            }),
        }
    }
}
