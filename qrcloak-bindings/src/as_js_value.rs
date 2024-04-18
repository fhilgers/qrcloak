use std::marker::PhantomData;

use serde::{Deserializer, Serializer};
use wasm_bindgen::JsValue;

pub struct AsJsValue<T>(PhantomData<T>);

impl<T> AsJsValue<T>
where
    T: for<'a> TryFrom<&'a JsValue> + Clone,
    for<'a> <T as TryFrom<&'a JsValue>>::Error: std::fmt::Debug,
    JsValue: From<T>,
{
    pub fn serialize<S>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        tsify_next::serde_wasm_bindgen::preserve::serialize(
            &JsValue::from(data.clone()),
            serializer,
        )
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<<T as ToOwned>::Owned, D::Error>
    where
        D: Deserializer<'de>,
    {
        let this: JsValue = tsify_next::serde_wasm_bindgen::preserve::deserialize(deserializer)?;
        let this = std::mem::ManuallyDrop::new(this);

        T::try_from(&this).map_err(|e| {
            serde::de::Error::custom(&format!("could not convert from js value: {:?}", e))
        })
    }
}

#[macro_export]
macro_rules! serde_impl {
    ($s:ident) => {
        impl Serialize for $s {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                crate::as_js_value::AsJsValue::<Self>::serialize(&self, serializer)
            }
        }

        impl<'de> Deserialize<'de> for $s {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                crate::as_js_value::AsJsValue::<Self>::deserialize(deserializer)
            }
        }
    };
}

#[macro_export]
macro_rules! wrapper_impl {
    ($from:path, $to:path) => {
        impl From<$to> for $from {
            fn from(s: $to) -> Self {
                Self(s)
            }
        }

        impl Into<$to> for $from {
            fn into(self) -> $to {
                self.0
            }
        }
    };
}
