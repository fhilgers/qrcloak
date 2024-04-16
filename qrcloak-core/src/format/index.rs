#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index {
    pub(crate) id: u32,
    pub(crate) index: u32,
    pub(crate) size: u32,
}

impl Index {
    pub fn is_head(&self) -> bool {
        self.index == 0
    }

    pub fn is_tail(&self) -> bool {
        !self.is_head()
    }
}
