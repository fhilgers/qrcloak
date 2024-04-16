use bytes::Bytes;

use super::{compression::CompressionSpec, encryption::EncryptionSpec};

#[derive(Debug, Clone)]
pub struct CompletePayload {
    pub(crate) data: Bytes,
    pub(crate) encryption: EncryptionSpec,
    pub(crate) compression: CompressionSpec,
}
