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

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

/// The payload format, being either a complete or partial payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// A complete payload that is not split accross multiple partial ones.
    Complete(CompletePayload),

    /// A partial payload that is split accross multiple partial ones.
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

#[cfg(feature = "uniffi")]
mod uniffi_bytes {
    use bytes::Bytes;

    use crate::UniffiCustomTypeConverter;

    uniffi::custom_type!(Bytes, Vec<u8>);

    impl UniffiCustomTypeConverter for Bytes {
        type Builtin = Vec<u8>;

        fn from_custom(obj: Self) -> Self::Builtin {
            obj.to_vec()
        }

        fn into_custom(val: Self::Builtin) -> uniffi::Result<Self>
        where
            Self: Sized,
        {
            Ok(val.into())
        }
    }
}
