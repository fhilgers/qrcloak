#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

/// The specification of the compression to be used for the payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
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
