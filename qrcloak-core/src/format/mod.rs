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

impl From<CompletePayload> for Payload {
    fn from(payload: CompletePayload) -> Self {
        Self::Complete(payload)
    }
}

impl From<PartialPayload> for Payload {
    fn from(payload: PartialPayload) -> Self {
        Self::Partial(payload)
    }
}

impl TryFrom<Payload> for CompletePayload {
    type Error = Payload;

    fn try_from(payload: Payload) -> Result<Self, Self::Error> {
        match payload {
            Payload::Complete(payload) => Ok(payload),
            p => Err(p),
        }
    }
}

impl TryFrom<Payload> for PartialPayload {
    type Error = Payload;

    fn try_from(payload: Payload) -> Result<Self, Self::Error> {
        match payload {
            Payload::Partial(payload) => Ok(payload),
            p => Err(p),
        }
    }
}

pub use complete::CompletePayload;
pub use compression::CompressionSpec;
pub use encryption::EncryptionSpec;
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
