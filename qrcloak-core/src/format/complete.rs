use bytes::Bytes;

use super::{encryption::EncryptionSpec, CompressionSpec};

#[cfg(feature = "json")]
use crate::format::base45::Base45IfHumanReadable;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

/// A complete payload, meaning a payload that is not split
/// accross multiple partial ones.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone)]
pub struct CompletePayload {
    /// The data of the payload.
    #[cfg_attr(feature = "serde", serde(with = "Base45IfHumanReadable"))]
    pub(crate) data: Bytes,

    /// The encryption to be used for the payload.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "EncryptionSpec::no_encryption", default)
    )]
    pub(crate) encryption: EncryptionSpec,

    /// The compression to be used for the payload.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "CompressionSpec::no_compression", default)
    )]
    pub(crate) compression: CompressionSpec,
}
