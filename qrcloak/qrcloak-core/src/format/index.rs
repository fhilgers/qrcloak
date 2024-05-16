#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

/// The index of a partial payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index {
    /// The id to match multiple partial payloads together
    pub(crate) id: u32,

    /// The index in the group
    pub(crate) index: u32,

    /// The total size of the group
    pub(crate) size: u32,
}

impl Index {
    /// Checks whether the index for the first
    /// element in the group
    pub fn is_head(&self) -> bool {
        self.index == 0
    }

    /// Checks whether the index for the remaining
    /// elements in the group
    pub fn is_tail(&self) -> bool {
        !self.is_head()
    }
}