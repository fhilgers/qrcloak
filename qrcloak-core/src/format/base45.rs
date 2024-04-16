#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "json")]
use schemars::JsonSchema;

#[cfg(feature = "serde")]
pub struct Base45IfHumanReadable;

#[cfg(feature = "serde")]
impl Base45IfHumanReadable {
    pub fn serialize<S>(data: &bytes::Bytes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            base45::encode(data).serialize(serializer)
        } else {
            data.serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bytes::Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            match base45::decode(<&str>::deserialize(deserializer)?) {
                Ok(buf) => Ok(buf.into()),
                Err(_) => Err(<D::Error as serde::de::Error>::invalid_value(
                    serde::de::Unexpected::Other("invalid base45 string"),
                    &"a valid base45 string",
                )),
            }
        } else {
            bytes::Bytes::deserialize(deserializer)
        }
    }
}

#[cfg(feature = "json")]
const BASE45_PATTERN: &str = "^[0-9A-Z\\s\\$%\\*\\+\\-\\.\\/:]*$";

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
                    pattern: Some(BASE45_PATTERN.into()),
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
