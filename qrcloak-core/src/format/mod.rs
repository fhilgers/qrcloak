mod base45;
mod complete;
mod compression;
mod encryption;
mod index;
mod partial;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use schemars::JsonSchema;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone)]
pub enum Payload {
    Complete(CompletePayload),
    Partial(PartialPayload),
}

pub use complete::CompletePayload;
pub use compression::{Compression, CompressionError, Decompression, DecompressionError};
pub use encryption::{
    AgeKeyDecryption, AgeKeyEncryption, AgePassphrase, Decryption, DecryptionError, Encryption,
    EncryptionError,
};
pub use index::Index;
pub use partial::{PartialPayload, PartialPayloadHead, PartialPayloadTail};

#[cfg(all(test, feature = "json"))]
mod tests {
    use insta::assert_json_snapshot;
    use schemars::schema_for;

    use crate::format::Payload;

    #[test]
    fn validate_schema() {
        let schema = schema_for!(Payload);

        assert_json_snapshot!(schema);
    }
}
