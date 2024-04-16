use bytes::Bytes;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

use super::base45::Base45IfHumanReadable;
use super::{index::Index, CompletePayload};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct PartialPayloadHead {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub(crate) complete: CompletePayload,
    pub(crate) index: Index,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct PartialPayloadTail {
    #[cfg_attr(feature = "serde", serde(with = "Base45IfHumanReadable"))]
    pub(crate) data: Bytes,
    pub(crate) index: Index,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub enum PartialPayload {
    Head(PartialPayloadHead),
    Tail(PartialPayloadTail),
}

impl PartialPayload {
    pub fn index(&self) -> Index {
        match self {
            PartialPayload::Head(head) => head.index,
            PartialPayload::Tail(tail) => tail.index,
        }
    }

    // TODO do not allow parsing of misconfigured
    pub fn is_misconfigured(&self) -> bool {
        match self {
            PartialPayload::Head(head) => head.index.is_tail(),
            PartialPayload::Tail(tail) => tail.index.is_head(),
        }
    }

    pub fn get_head(&self) -> Option<&PartialPayloadHead> {
        match self {
            PartialPayload::Head(head) => Some(head),
            _ => None,
        }
    }

    pub fn get_tail(&self) -> Option<&PartialPayloadTail> {
        match self {
            PartialPayload::Tail(tail) => Some(tail),
            _ => None,
        }
    }

    pub fn get_head_mut(&mut self) -> Option<&mut PartialPayloadHead> {
        match self {
            PartialPayload::Head(head) => Some(head),
            _ => None,
        }
    }

    pub fn get_tail_mut(&mut self) -> Option<&PartialPayloadTail> {
        match self {
            PartialPayload::Tail(tail) => Some(tail),
            _ => None,
        }
    }
}
