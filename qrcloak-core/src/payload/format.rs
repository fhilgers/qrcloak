use std::io::Read;

use age::{secrecy::SecretString, x25519, DecryptError, Identity};
#[cfg(feature = "json")]
use schemars::JsonSchema;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletePayload {
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "PayloadMetadata::is_empty", default)
    )]
    pub payload_metadata: PayloadMetadata,

    #[cfg_attr(feature = "serde", serde(with = "Base45IfHumanReadable"))]
    pub data: Vec<u8>,
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

#[cfg(feature = "serde")]
struct Base45IfHumanReadable;

#[cfg(feature = "serde")]
impl Base45IfHumanReadable {
    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            base45::encode(data).serialize(serializer)
        } else {
            data.serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            match base45::decode(<&str>::deserialize(deserializer)?) {
                Ok(buf) => Ok(buf),
                Err(_) => Err(<D::Error as serde::de::Error>::invalid_value(
                    serde::de::Unexpected::Other("invalid base45 string"),
                    &"a valid base45 string",
                )),
            }
        } else {
            Vec::<u8>::deserialize(deserializer)
        }
    }
}

#[cfg(feature = "json")]
impl JsonSchema for Base45IfHumanReadable {
    fn schema_name() -> String {
        "String".to_string()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            format: Some("base45".into()),
            string: Some(
                schemars::schema::StringValidation {
                    pattern: Some("^[0-9A-Z\\s\\$%\\*\\+\\-\\.\\/:]*$".into()),
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        }
        .into()
    }

    fn is_referenceable() -> bool {
        false
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PayloadMetadata {
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub encryption: Option<EncryptionSpec>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub compression: Option<CompressionSpec>,
}

#[cfg(feature = "serde")]
impl PayloadMetadata {
    fn is_empty(&self) -> bool {
        self.encryption.is_none() && self.compression.is_none()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialPayload {
    pub index_metadata: IndexMetadata,
    pub data: PartialPayloadData,
}

impl From<Payload> for PartialPayload {
    fn from(value: Payload) -> Self {
        match value {
            Payload::Complete(c) => c.split(1).pop().unwrap(),
            Payload::Partial(p) => p,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexMetadata {
    pub id: u32,
    pub index: u32,
    pub size: u32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(tag = "type"),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionSpec {
    AgeKey(#[cfg_attr(feature = "serde", serde(skip_serializing, default))] AgeKeySpec),
    AgePassword(#[cfg_attr(feature = "serde", serde(skip_serializing, default))] AgePasswordSpec),
    NoEncryption,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgeKeySpec;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgePasswordSpec;

impl AgeKeySpec {
    pub fn decrypt(
        &self,
        key: &x25519::Identity,
        data: &[u8],
    ) -> Result<Vec<u8>, age::DecryptError> {
        let key: &dyn Identity = key;

        let decryptor = match age::Decryptor::new(data)? {
            age::Decryptor::Recipients(c) => Ok(c),
            age::Decryptor::Passphrase(_) => Err(DecryptError::NoMatchingKeys),
        }?;

        let mut decrypted_data = Vec::with_capacity(data.len());

        let mut writer = decryptor.decrypt([key].into_iter())?;
        writer.read_to_end(&mut decrypted_data)?;

        Ok(decrypted_data)
    }
}

impl AgePasswordSpec {
    pub fn decrypt(&self, passphrase: &str, data: &[u8]) -> Result<Vec<u8>, age::DecryptError> {
        let decryptor = match age::Decryptor::new(data)? {
            age::Decryptor::Recipients(_) => Err(DecryptError::NoMatchingKeys),
            age::Decryptor::Passphrase(c) => Ok(c),
        }?;

        let mut decrypted_data = Vec::with_capacity(data.len());

        let mut writer = decryptor.decrypt(&SecretString::new(passphrase.into()), None)?;
        writer.read_to_end(&mut decrypted_data)?;

        Ok(decrypted_data)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(tag = "type"),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionSpec {
    Gzip,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(tag = "type"),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "json", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartialPayloadData {
    Head(CompletePayload),
    Tail {
        #[cfg_attr(feature = "serde", serde(with = "Base45IfHumanReadable"))]
        data: Vec<u8>,
    },
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use insta::assert_json_snapshot;
    use schemars::schema_for;

    use crate::payload::Payload;

    #[test]
    fn validate_schema() {
        let schema = schema_for!(Payload);

        assert_json_snapshot!(schema);
    }
}
