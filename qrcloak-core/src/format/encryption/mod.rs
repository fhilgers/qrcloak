use bytes::Bytes;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

use super::CompletePayload;

mod age_encryption;

pub use age_encryption::{AgeKeyDecryption, AgeKeyEncryption, AgePassphrase};

#[derive(Debug, Clone, Default)]
pub enum Encryption {
    #[default]
    NoEncryption,
    AgePasshprase(age_encryption::AgePassphrase),
    AgeKey(age_encryption::AgeKeyEncryption),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, Default)]
pub enum EncryptionSpec {
    #[default]
    NoEncryption,
    AgePassphrase,
    AgeKey,
}

impl EncryptionSpec {
    pub fn no_encryption(&self) -> bool {
        match self {
            EncryptionSpec::NoEncryption => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Decryption {
    #[default]
    NoEncryption,
    AgePassphrase(age_encryption::AgePassphrase),
    AgeKey(age_encryption::AgeKeyDecryption),
}

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error(transparent)]
    Age(#[from] age::EncryptError),
}

#[derive(Debug, Error)]
pub enum DecryptionError {
    #[error("tried to decrypt with {tried:?} but payload is decrypted with {payload:?}")]
    SpecMismtach {
        payload: EncryptionSpec,
        tried: Decryption,
    },

    #[error(transparent)]
    Age(#[from] age::DecryptError),
}

impl Encryption {
    pub fn process(&self, data: Bytes) -> Result<Bytes, EncryptionError> {
        match self {
            Encryption::NoEncryption => Ok(data),
            Encryption::AgePasshprase(pw) => Ok(pw.encrypt(data)?),
            Encryption::AgeKey(key) => Ok(key.encrypt(data)?),
        }
    }

    pub fn spec(&self) -> EncryptionSpec {
        match self {
            Encryption::NoEncryption => EncryptionSpec::NoEncryption,
            Encryption::AgePasshprase(_) => EncryptionSpec::AgePassphrase,
            Encryption::AgeKey(_) => EncryptionSpec::AgeKey,
        }
    }
}

impl Decryption {
    pub fn process(&self, data: &mut CompletePayload) -> Result<(), DecryptionError> {
        match (self, &data.encryption) {
            (Decryption::NoEncryption, EncryptionSpec::NoEncryption) => Ok(()),
            (Decryption::AgePassphrase(pw), EncryptionSpec::AgePassphrase) => {
                data.data = pw.decrypt(data.data.clone())?;
                Ok(())
            }
            (Decryption::AgeKey(key), EncryptionSpec::AgeKey) => {
                data.data = key.decrypt(data.data.clone())?;
                Ok(())
            }
            (tried, payload) => Err(DecryptionError::SpecMismtach {
                payload: payload.clone(),
                tried: tried.clone(),
            }),
        }
    }
}
