use bytes::Bytes;

use super::{index::Index, CompletePayload};

#[derive(Debug, Clone)]
pub struct PartialPayloadHead {
    pub(crate) complete: CompletePayload,
    pub(crate) index: Index,
}

#[derive(Debug, Clone)]
pub struct PartialPayloadTail {
    pub(crate) data: Bytes,
    pub(crate) index: Index,
}

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
