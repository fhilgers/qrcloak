#[cfg(feature = "json")]
mod decoder;

#[cfg(feature = "json")]
mod encoder;

mod compression;
mod encryption;
mod extract;
mod generate;
mod merge;
mod split;
mod utils;

#[cfg(feature = "json")]
pub use decoder::{Decoder, DecodingError, DecodingOpts};
#[cfg(feature = "json")]
pub use encoder::{Encoder, EncodingError, EncodingOpts};

pub use compression::{Compression, CompressionError, Decompression, DecompressionError};
pub use encryption::{
    AgeKeyDecryption, AgeKeyEncryption, AgePassphrase, Decryption, DecryptionError, Encryption,
    EncryptionError,
};
pub use extract::{PayloadExtractionError, PayloadExtractor};
pub use generate::{PayloadGenerationError, PayloadGenerator};
pub use merge::{MergeResult, PayloadMerger};
pub use split::PayloadSplitter;

pub enum OneOrMany<T> {
    Empty,
    One(T),
    Many(Vec<T>),
}

#[cfg(feature = "json")]
impl<T: serde::Serialize> serde::Serialize for OneOrMany<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let empty: [T; 0] = [];
        match self {
            OneOrMany::Empty => empty.serialize(serializer),
            OneOrMany::One(v) => v.serialize(serializer),
            OneOrMany::Many(v) => v.serialize(serializer),
        }
    }
}

#[cfg(feature = "json")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for OneOrMany<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum OneOrManyInner<T> {
            One(T),
            Many(Vec<T>),
        }

        match OneOrManyInner::<T>::deserialize(deserializer)? {
            OneOrManyInner::One(value) => Ok(OneOrMany::One(value)),
            OneOrManyInner::Many(value) => {
                if value.is_empty() {
                    Ok(OneOrMany::Empty)
                } else {
                    Ok(OneOrMany::Many(value))
                }
            }
        }
    }
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(value: Vec<T>) -> Self {
        if value.is_empty() {
            Self::Empty
        } else if value.len() == 1 {
            Self::One(value.into_iter().next().unwrap())
        } else {
            Self::Many(value)
        }
    }
}

impl<T> FromIterator<T> for OneOrMany<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

#[cfg(all(test, feature = "json"))]
mod tests {

    use bytes::Bytes;

    use crate::payload::{
        extract::PayloadExtractor,
        merge::{MergeResult, PayloadMerger},
        AgeKeyDecryption, AgeKeyEncryption, Decoder, Decryption, Encoder, Encryption,
    };

    use super::{generate::PayloadGenerator, split::PayloadSplitter};

    #[test]
    fn test_roundtrip() {
        let expected = b"hello world";

        let payload = PayloadGenerator::default()
            .generate(Bytes::from_static(expected))
            .expect("should generate");

        let split = PayloadSplitter::default().with_splits(4).split(payload);

        let encoded = Encoder::default()
            .with_encoding(crate::payload::EncodingOpts::Json {
                pretty: true,
                merge: true,
            })
            .encode(split)
            .expect("should encode");

        let decoded = Decoder::default()
            .with_opts(crate::payload::DecodingOpts::Json)
            .decode(encoded[0].as_bytes())
            .expect("should decode");

        let MergeResult(mut complete, _) = PayloadMerger::default().merge(decoded);

        assert_eq!(complete.len(), 1);

        let complete = complete.pop().expect("should have one complete");

        let data = PayloadExtractor::default()
            .extract(complete)
            .expect("should extract");

        assert_eq!(&*data, expected);
    }

    #[test]
    fn test_roundtrip_age_keys() {
        let expected = b"hello world";
        let identity = age::x25519::Identity::generate();

        let payload = PayloadGenerator::default()
            .with_encryption(Encryption::AgeKey(AgeKeyEncryption::new(vec![
                identity.to_public()
            ])))
            .generate(Bytes::from_static(expected))
            .expect("should generate");

        let split = PayloadSplitter::default().with_splits(4).split(payload);

        let MergeResult(mut complete, _) = PayloadMerger::default().merge(split);

        assert_eq!(complete.len(), 1);

        let complete = complete.pop().expect("should have one complete");

        let data = PayloadExtractor::default()
            .with_decryption(Decryption::AgeKey(AgeKeyDecryption::new(vec![identity])))
            .extract(complete)
            .expect("should extract");

        assert_eq!(&*data, expected);
    }
}
