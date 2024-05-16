use std::str::FromStr;

use qrcloak_core::secrecy::SecretString;
use qrcloak_core::x25519;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use tsify_next::Tsify;

use uniffi::Enum;
use uniffi::Error;

use uniffi::Object;

use wasm_bindgen::prelude::*;
use wasm_bindgen_brand::Brand;
use wasm_bindgen_derive::TryFromJsValue;

use crate::serde_impl;
use crate::uniffi_object_clone;
use crate::wrapper_impl;

#[derive(Debug, thiserror::Error, Error)]
#[uniffi(flat_error)]
pub enum ParsingError {
    #[error("invalid identity: {0}")]
    Identity(String),

    #[error("invalid recipient: {0}")]
    Recipient(String),
}

impl Into<JsValue> for ParsingError {
    fn into(self) -> JsValue {
        JsValue::from(self.to_string())
    }
}

#[derive(TryFromJsValue, Clone, Brand, Object)]
#[wasm_bindgen]
pub struct AgeIdentity(x25519::Identity);

#[uniffi::export]
#[wasm_bindgen]
impl AgeIdentity {
    #[uniffi::constructor]
    pub fn try_from_string(string: String) -> Result<AgeIdentity, ParsingError> {
        let x = x25519::Identity::from_str(&string)
            .map_err(|e| ParsingError::Identity(e.to_string()))?;

        Ok(Self(x))
    }

    #[uniffi::constructor]
    pub fn generate() -> Self {
        Self(x25519::Identity::generate())
    }

    pub fn to_public(&self) -> AgeRecipient {
        AgeRecipient(self.0.to_public())
    }
}

#[derive(TryFromJsValue, Clone, Brand, Object)]
#[wasm_bindgen]
pub struct AgeRecipient(x25519::Recipient);

#[uniffi::export]
#[wasm_bindgen]
impl AgeRecipient {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn try_from_string(string: String) -> Result<AgeRecipient, ParsingError> {
        let x = x25519::Recipient::from_str(&string)
            .map_err(|e| ParsingError::Recipient(e.to_string()))?;

        Ok(Self(x))
    }
}

#[derive(TryFromJsValue, Clone, Object)]
#[wasm_bindgen]
pub struct Passphrase(SecretString);

#[uniffi::export]
#[wasm_bindgen]
impl Passphrase {
    #[uniffi::constructor]
    #[wasm_bindgen(constructor)]
    pub fn new(passphrase: String) -> Self {
        Self(SecretString::new(passphrase))
    }
}

serde_impl!(AgeIdentity);
serde_impl!(AgeRecipient);
serde_impl!(Passphrase);

wrapper_impl!(AgeIdentity, x25519::Identity);
wrapper_impl!(AgeRecipient, x25519::Recipient);
wrapper_impl!(Passphrase, SecretString);

uniffi_object_clone!(Passphrase);
uniffi_object_clone!(AgeRecipient);
uniffi_object_clone!(AgeIdentity);

#[derive(Tsify, Serialize, Deserialize, Enum)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Encryption {
    NoEncryption,
    AgePassphrase { passphrase: Passphrase },
    AgeKey { recipients: Vec<AgeRecipient> },
}

impl From<Encryption> for qrcloak_core::payload::Encryption {
    fn from(encryption: Encryption) -> Self {
        match encryption {
            Encryption::NoEncryption => qrcloak_core::payload::Encryption::NoEncryption,
            Encryption::AgePassphrase { passphrase } => {
                qrcloak_core::payload::Encryption::AgePassphrase(
                    qrcloak_core::payload::AgePassphrase::from(passphrase.0),
                )
            }
            Encryption::AgeKey { recipients } => qrcloak_core::payload::Encryption::AgeKey(
                qrcloak_core::payload::AgeKeyEncryption::from(
                    recipients.into_iter().map(|x| x.0).collect::<Vec<_>>(),
                ),
            ),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Enum)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Decryption {
    NoEncryption,
    AgePassphrase { passphrase: Passphrase },
    AgeKey { identities: Vec<AgeIdentity> },
}

impl From<Decryption> for qrcloak_core::payload::Decryption {
    fn from(decryption: Decryption) -> Self {
        match decryption {
            Decryption::NoEncryption => qrcloak_core::payload::Decryption::NoEncryption,
            Decryption::AgePassphrase { passphrase } => {
                qrcloak_core::payload::Decryption::AgePassphrase(
                    qrcloak_core::payload::AgePassphrase::from(passphrase.0),
                )
            }
            Decryption::AgeKey { identities } => qrcloak_core::payload::Decryption::AgeKey(
                qrcloak_core::payload::AgeKeyDecryption::from(
                    identities.into_iter().map(|x| x.0).collect::<Vec<_>>(),
                ),
            ),
        }
    }
}
