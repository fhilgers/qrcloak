#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

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
