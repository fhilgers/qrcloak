use std::fmt::Debug;

use age::{
    secrecy::{ExposeSecret, SecretString},
    DecryptError,
};
use thiserror::Error;

use super::format::{CompletePayload, EncryptionSpec};

#[derive(Clone, Default)]
pub enum DecryptionOpts {
    AgeKey(age::x25519::Identity),
    AgePassword(SecretString),
    #[default]
    NoEncryption,
}

impl Debug for DecryptionOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecryptionOpts::AgeKey(key) => write!(f, "AgeKey({:?})", key.to_string()),
            DecryptionOpts::AgePassword(passphrase) => write!(f, "AgePassword({:?})", passphrase),
            DecryptionOpts::NoEncryption => write!(f, "NoEncryption"),
        }
    }
}

#[derive(Debug, Error)]
#[error("decryption opts do not match payload (have {have:?}, wanted {wanted:?})")]
pub struct EncryptionMismatch {
    have: DecryptionOpts,
    wanted: Option<EncryptionSpec>,
}

impl DecryptionOpts {
    pub fn decrypt(&self, payload: &CompletePayload) -> Result<Vec<u8>, ExtractionError> {
        self.matches(payload)?;

        let data = &payload.data;
        match (self, &payload.payload_metadata.encryption) {
            (DecryptionOpts::AgeKey(key), Some(EncryptionSpec::AgeKey(spec))) => {
                Ok(spec.decrypt(key, data)?)
            }
            (DecryptionOpts::AgePassword(passphrase), Some(EncryptionSpec::AgePassword(spec))) => {
                Ok(spec.decrypt(passphrase.expose_secret(), data)?)
            }
            (DecryptionOpts::NoEncryption, None) => Ok(data.to_vec()),
            (have, wanted) => {
                panic!("decryption opts do not match payload (have {have:?}, wanted {wanted:?})")
            }
        }
    }

    pub fn matches(&self, payload: &CompletePayload) -> Result<(), EncryptionMismatch> {
        match (self, &payload.payload_metadata.encryption) {
            (DecryptionOpts::AgeKey(_), Some(EncryptionSpec::AgeKey(_))) => Ok(()),
            (DecryptionOpts::AgePassword(_), Some(EncryptionSpec::AgePassword(_))) => Ok(()),
            (DecryptionOpts::NoEncryption, None) => Ok(()),
            (have, wanted) => Err(EncryptionMismatch {
                have: have.clone(),
                wanted: wanted.clone(),
            }),
        }
    }
}

#[derive(Default)]
pub struct PayloadExtractor {
    decryption_opts: DecryptionOpts,
}

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("transparent")]
    DecryptionError(#[from] DecryptError),

    #[error("transparent")]
    EncryptionMismatch(#[from] EncryptionMismatch),
}

impl PayloadExtractor {
    pub fn with_decryption_opts(mut self, decryption_opts: DecryptionOpts) -> Self {
        self.decryption_opts = decryption_opts;
        self
    }

    pub fn extract(&self, payload: &CompletePayload) -> Result<Vec<u8>, ExtractionError> {
        self.decryption_opts.decrypt(payload)
    }
}
