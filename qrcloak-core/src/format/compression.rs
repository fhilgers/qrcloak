#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

/// The specification of the compression to be used for the payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CompressionSpec {
    /// No compression is used.
    #[default]
    NoCompression,
    // TODO: Add compression specs
    Gzip,
}

impl CompressionSpec {
    pub(crate) fn no_compression(&self) -> bool {
        match self {
            CompressionSpec::NoCompression => true,
            _ => false,
        }
    }
}
