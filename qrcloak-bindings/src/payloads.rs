use std::ops::{Deref, DerefMut};

use qrcloak_core::format::{CompletePayload, PartialPayload, Payload};

use crate::UniffiCustomTypeConverter;

#[derive(tsify_next::Tsify, serde::Serialize, serde::Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct Payloads(Vec<Payload>);

impl UniffiCustomTypeConverter for Payloads {
    type Builtin = Vec<Payload>;

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.0
    }

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(val))
    }
}

uniffi::custom_type!(Payloads, Vec<Payload>);

impl<T: Into<Payload>> FromIterator<T> for Payloads {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<Payload>> for Payloads {
    fn from(value: Vec<Payload>) -> Self {
        Self(value)
    }
}

impl From<Vec<PartialPayload>> for Payloads {
    fn from(value: Vec<PartialPayload>) -> Self {
        value.into_iter().collect()
    }
}

impl From<Vec<CompletePayload>> for Payloads {
    fn from(value: Vec<CompletePayload>) -> Self {
        value.into_iter().collect()
    }
}

impl IntoIterator for Payloads {
    type Item = Payload;
    type IntoIter = std::vec::IntoIter<Payload>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Into<Vec<Payload>> for Payloads {
    fn into(self) -> Vec<Payload> {
        self.0
    }
}

impl Deref for Payloads {
    type Target = Vec<Payload>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Payloads {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
