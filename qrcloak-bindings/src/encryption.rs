use std::str::FromStr;

use qrcloak_core::secrecy::SecretString;
use qrcloak_core::x25519;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use tsify_next::Tsify;

use wasm_bindgen::JsError;

use wasm_bindgen::prelude::*;
use wasm_bindgen_brand::Brand;
use wasm_bindgen_derive::TryFromJsValue;

use crate::serde_impl;
use crate::wrapper_impl;

#[derive(TryFromJsValue, Clone, Brand)]
#[wasm_bindgen]
pub struct AgeIdentity(x25519::Identity);

#[wasm_bindgen]
impl AgeIdentity {
    pub fn try_from_string(string: String) -> Result<AgeIdentity, JsError> {
        let x = x25519::Identity::from_str(&string)
            .map_err(|e| JsError::new(&format!("invalid recipient: {}", e)))?;

        Ok(Self(x))
    }

    pub fn generate() -> Self {
        Self(x25519::Identity::generate())
    }

    pub fn to_public(&self) -> AgeRecipient {
        AgeRecipient(self.0.to_public())
    }
}

#[derive(TryFromJsValue, Clone, Brand)]
#[wasm_bindgen]
pub struct AgeRecipient(x25519::Recipient);

#[wasm_bindgen]
impl AgeRecipient {
    pub fn try_from_string(string: String) -> Result<AgeRecipient, JsError> {
        let x = x25519::Recipient::from_str(&string)
            .map_err(|e| JsError::new(&format!("invalid recipient: {}", e)))?;

        Ok(Self(x))
    }
}

#[derive(TryFromJsValue, Clone)]
#[wasm_bindgen]
pub struct AgePassphrase(SecretString);

#[wasm_bindgen]
impl AgePassphrase {
    #[wasm_bindgen(constructor)]
    pub fn new(passphrase: String) -> Self {
        Self(SecretString::new(passphrase))
    }
}

serde_impl!(AgeIdentity);
serde_impl!(AgeRecipient);
serde_impl!(AgePassphrase);

wrapper_impl!(AgeIdentity, x25519::Identity);
wrapper_impl!(AgeRecipient, x25519::Recipient);
wrapper_impl!(AgePassphrase, SecretString);

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Encryption {
    NoEncryption,
    AgePassphrase(AgePassphrase),
    AgeKey(Vec<AgeRecipient>),
}

impl From<Encryption> for qrcloak_core::payload::Encryption {
    fn from(encryption: Encryption) -> Self {
        match encryption {
            Encryption::NoEncryption => qrcloak_core::payload::Encryption::NoEncryption,
            Encryption::AgePassphrase(passphrase) => {
                qrcloak_core::payload::Encryption::AgePassphrase(
                    qrcloak_core::payload::AgePassphrase::from(passphrase.0),
                )
            }
            Encryption::AgeKey(recipients) => qrcloak_core::payload::Encryption::AgeKey(
                qrcloak_core::payload::AgeKeyEncryption::from(
                    recipients.into_iter().map(|x| x.0).collect::<Vec<_>>(),
                ),
            ),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Decryption {
    NoEncryption,
    AgePassphrase(AgePassphrase),
    AgeKey(Vec<AgeIdentity>),
}

impl From<Decryption> for qrcloak_core::payload::Decryption {
    fn from(decryption: Decryption) -> Self {
        match decryption {
            Decryption::NoEncryption => qrcloak_core::payload::Decryption::NoEncryption,
            Decryption::AgePassphrase(passphrase) => {
                qrcloak_core::payload::Decryption::AgePassphrase(
                    qrcloak_core::payload::AgePassphrase::from(passphrase.0),
                )
            }
            Decryption::AgeKey(identities) => qrcloak_core::payload::Decryption::AgeKey(
                qrcloak_core::payload::AgeKeyDecryption::from(
                    identities.into_iter().map(|x| x.0).collect::<Vec<_>>(),
                ),
            ),
        }
    }
}
