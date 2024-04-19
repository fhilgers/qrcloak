use bytes::Bytes;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

#[cfg(feature = "serde")]
use super::base45::Base45IfHumanReadable;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

use super::{index::Index, CompressionSpec, EncryptionSpec};

/// A partial payload head, meaning the first partial
/// payload in a group.
/// This payload carries additional information about
/// encryption and compression.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialPayloadHead {
    /// The data of the payload.
    #[cfg_attr(feature = "serde", serde(with = "Base45IfHumanReadable"))]
    #[cfg_attr(feature = "wasm", tsify(type = "string"))]
    pub(crate) data: Bytes,

    /// The encryption to be used for the group of partial payloads.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "EncryptionSpec::no_encryption", default)
    )]
    pub(crate) encryption: EncryptionSpec,

    /// The compression to be used for the group of partial payloads.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "CompressionSpec::no_compression", default)
    )]
    pub(crate) compression: CompressionSpec,

    /// The index of the payload.
    pub(crate) index: Index,
}

/// A partial payload tail, meaning any partial payload
/// that is not the head (the first).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialPayloadTail {
    /// The data of the payload.
    #[cfg_attr(feature = "serde", serde(with = "Base45IfHumanReadable"))]
    #[cfg_attr(feature = "wasm", tsify(type = "string"))]
    pub(crate) data: Bytes,

    /// The index of the payload.
    pub(crate) index: Index,
}

/// A partial payload
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartialPayload {
    /// The head of the group of partial payloads.
    Head(PartialPayloadHead),
    /// The following partial payloads in the group.
    Tail(PartialPayloadTail),
}

impl PartialPayload {
    /// Returns the index of the payload.
    pub fn index(&self) -> Index {
        match self {
            PartialPayload::Head(head) => head.index,
            PartialPayload::Tail(tail) => tail.index,
        }
    }

    // TODO do not allow parsing of misconfigured
    /// Returns `true` if the payload is misconfigured.
    /// This means that either
    /// - the index of the head is not valid for the head
    /// - the index of the tail is not valid for the tail
    pub fn is_misconfigured(&self) -> bool {
        match self {
            PartialPayload::Head(head) => head.index.is_tail(),
            PartialPayload::Tail(tail) => tail.index.is_head(),
        }
    }

    /// Get a reference to the head if the payload is a head.
    pub fn get_head(&self) -> Option<&PartialPayloadHead> {
        match self {
            PartialPayload::Head(head) => Some(head),
            _ => None,
        }
    }

    /// Get a reference to the tail if the payload is a tail.
    pub fn get_tail(&self) -> Option<&PartialPayloadTail> {
        match self {
            PartialPayload::Tail(tail) => Some(tail),
            _ => None,
        }
    }

    /// Get a mutable reference to the head if the payload is a head.
    pub fn get_head_mut(&mut self) -> Option<&mut PartialPayloadHead> {
        match self {
            PartialPayload::Head(head) => Some(head),
            _ => None,
        }
    }

    /// Get a mutable reference to the tail if the payload is a tail.
    pub fn get_tail_mut(&mut self) -> Option<&PartialPayloadTail> {
        match self {
            PartialPayload::Tail(tail) => Some(tail),
            _ => None,
        }
    }
}
