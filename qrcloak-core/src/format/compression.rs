#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

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
        }
    }
}
