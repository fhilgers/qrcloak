#[cfg(json)]
use schemars::JsonSchema;

#[cfg(serde)]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg_attr(serde, derive(Serialize, Deserialize))]
#[cfg_attr(json, JsonSchema)]
#[cfg_attr(serde, serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Complete(CompletePayload),
    Partial(PartialPayload),
}

#[cfg_attr(serde, derive(Serialize, Deserialize))]
#[cfg_attr(json, JsonSchema)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletePayload {
    pub payload_metadata: PayloadMetadata,

    #[cfg_attr(serde, serde(with = "Base45IfHumanReadable"))]
    pub data: Vec<u8>,
}

#[cfg(serde)]
struct Base45IfHumanReadable;

#[cfg(serde)]
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

#[cfg(json)]
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

#[cfg_attr(serde, derive(Serialize, Deserialize))]
#[cfg_attr(json, JsonSchema)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadMetadata {
    pub encryption: Option<EncryptionSpec>,
    pub compression: Option<CompressionSpec>,
}

#[cfg_attr(serde, derive(Serialize, Deserialize))]
#[cfg_attr(json, JsonSchema)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialPayload {
    pub index_metadata: IndexMetadata,
    pub data: PartialPayloadData,
}

#[cfg_attr(serde, derive(Serialize, Deserialize))]
#[cfg_attr(json, JsonSchema)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexMetadata {
    pub id: u32,
    pub index: u32,
    pub size: u32,
}

#[cfg_attr(serde, derive(Serialize, Deserialize))]
#[cfg_attr(serde, serde(tag = "type"))]
#[cfg_attr(json, JsonSchema)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionSpec {
    AgeKey,
    AgePassword,
    NoEncryption,
}

#[cfg_attr(serde, derive(Serialize, Deserialize))]
#[cfg_attr(serde, serde(tag = "type"))]
#[cfg_attr(json, JsonSchema)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionSpec {
    Gzip,
}

#[cfg_attr(serde, derive(Serialize, Deserialize))]
#[cfg_attr(serde, serde(tag = "type"))]
#[cfg_attr(json, JsonSchema)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartialPayloadData {
    Head(CompletePayload),
    Tail(#[cfg_attr(serde, serde(with = "Base45IfHumanReadable"))] Vec<u8>),
}
