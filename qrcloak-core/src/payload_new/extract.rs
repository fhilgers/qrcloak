use bytes::Bytes;
use thiserror::Error;

use crate::format::{
    CompletePayload, Decompression, DecompressionError, Decryption, DecryptionError,
};

#[derive(Default, Clone)]
pub struct PayloadExtractor {
    decryption: Decryption,
    decompression: Decompression,
}

#[derive(Debug, Error)]
pub enum PayloadExtractionError {
    #[error(transparent)]
    DecryptionError(#[from] DecryptionError),

    #[error(transparent)]
    DecompressionError(#[from] DecompressionError),
}

impl PayloadExtractor {
    pub fn with_decryption(mut self, decryption: Decryption) -> Self {
        self.decryption = decryption;
        self
    }

    pub fn with_decompression(mut self, decompression: Decompression) -> Self {
        self.decompression = decompression;
        self
    }

    pub fn extract(&self, mut payload: CompletePayload) -> Result<Bytes, PayloadExtractionError> {
        self.decryption.process(&mut payload)?;
        self.decompression.process(&mut payload)?;

        Ok(payload.data)
    }
}
