use bytes::Bytes;
use thiserror::Error;

use crate::format::CompletePayload;

use super::{Compression, CompressionError, Encryption, EncryptionError};
#[derive(Default, Clone)]
pub struct PayloadGenerator {
    encryption: Encryption,
    compression: Compression,
}

#[derive(Debug, Error)]
pub enum PayloadGenerationError {
    #[error(transparent)]
    EncryptionError(#[from] EncryptionError),
    #[error(transparent)]
    CompressionError(#[from] CompressionError),
}

impl PayloadGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_encryption(mut self, encryption: Encryption) -> Self {
        self.encryption = encryption;
        self
    }

    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    pub fn generate(&self, data: Bytes) -> Result<CompletePayload, PayloadGenerationError> {
        let compressed = self.compression.process(data)?;
        let encrypted = self.encryption.process(compressed)?;

        Ok(CompletePayload {
            data: encrypted,
            encryption: self.encryption.spec(),
            compression: self.compression.spec(),
        })
    }
}
