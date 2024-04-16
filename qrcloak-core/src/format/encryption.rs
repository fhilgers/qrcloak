#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

/// The specification of the encryption to be used for the payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum EncryptionSpec {
    /// No encryption is used.
    #[default]
    NoEncryption,

    /// The payload is encrypted with a passphrase using [`age`].
    AgePassphrase,

    /// The payload is encrypted with a x25519 key using [`age`].
    AgeKey,
}

impl EncryptionSpec {
    pub(crate) fn no_encryption(&self) -> bool {
        match self {
            EncryptionSpec::NoEncryption => true,
            _ => false,
        }
    }
}
