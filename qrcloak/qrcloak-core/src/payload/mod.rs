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

pub use compression::{
    Compression, CompressionError, Decompression, DecompressionError, GzipCompression,
};
pub use encryption::{
    AgeKeyDecryption, AgeKeyEncryption, AgePassphrase, Decryption, DecryptionError, Encryption,
    EncryptionError,
};
pub use extract::{PayloadExtractionError, PayloadExtractor};
pub use generate::{PayloadGenerationError, PayloadGenerator};
pub use merge::{MergeResult, PayloadMerger, UnmergedPayloads};
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

impl<T> Into<Vec<T>> for OneOrMany<T> {
    fn into(self) -> Vec<T> {
        match self {
            OneOrMany::Empty => vec![],
            OneOrMany::One(v) => vec![v],
            OneOrMany::Many(v) => v,
        }
    }
}

#[cfg(all(test, feature = "json"))]
mod tests {

    use age::{secrecy::SecretString, x25519, DecryptError};
    use bytes::Bytes;
    use thiserror::Error;

    use crate::{
        format::{CompletePayload, CompressionSpec, Payload},
        payload::{
            extract::PayloadExtractor, merge::PayloadMerger, AgeKeyDecryption, AgeKeyEncryption,
            Decoder, Decryption, Encoder, Encryption,
        },
    };

    use super::{
        generate::PayloadGenerator, split::PayloadSplitter, AgePassphrase, Compression,
        DecodingError, DecodingOpts, Decompression, DecompressionError, DecryptionError,
        EncodingError, EncodingOpts, GzipCompression, PayloadExtractionError,
        PayloadGenerationError,
    };

    #[derive(Debug, Error)]
    enum RoundtripError {
        #[error(transparent)]
        GenerationError(#[from] PayloadGenerationError),
        #[error(transparent)]
        ExtractionError(#[from] PayloadExtractionError),
        #[error(transparent)]
        DecodingError(#[from] DecodingError),
        #[error(transparent)]
        EncodingError(#[from] EncodingError),
    }

    #[derive(Debug, Default)]
    struct TesterBuilder {
        encryption: Option<Encryption>,
        compression: Option<Compression>,
        decryption: Option<Decryption>,
        decompression: Option<Decompression>,
        encoding: Option<EncodingOpts>,
        decoding: Option<DecodingOpts>,
        splits: Option<u32>,
    }

    impl TesterBuilder {
        fn with_encryption(mut self, encryption: Option<Encryption>) -> Self {
            self.encryption = encryption;
            self
        }

        fn with_compression(mut self, compression: Option<Compression>) -> Self {
            self.compression = compression;
            self
        }

        fn with_decryption(mut self, decryption: Option<Decryption>) -> Self {
            self.decryption = decryption;
            self
        }

        fn with_decompression(mut self, decompression: Option<Decompression>) -> Self {
            self.decompression = decompression;
            self
        }

        fn with_splits(mut self, splits: Option<u32>) -> Self {
            self.splits = splits;
            self
        }

        fn with_encoding(mut self, encoding: Option<EncodingOpts>) -> Self {
            self.encoding = encoding;
            self
        }

        fn with_decoding(mut self, decoding: Option<DecodingOpts>) -> Self {
            self.decoding = decoding;
            self
        }

        fn build(self) -> Tester {
            let mut generator = PayloadGenerator::default();

            if let Some(encryption) = self.encryption {
                generator = generator.with_encryption(encryption);
            }

            if let Some(compression) = self.compression {
                generator = generator.with_compression(compression);
            }

            let mut splitter = PayloadSplitter::default();

            if let Some(splits) = self.splits {
                splitter = splitter.with_splits(splits);
            }

            let mut extractor = PayloadExtractor::default();

            if let Some(decryption) = self.decryption {
                extractor = extractor.with_decryption(decryption);
            }

            if let Some(decompression) = self.decompression {
                extractor = extractor.with_decompression(decompression);
            }

            let merger = PayloadMerger::default();

            let mut encoder = Encoder::new();

            if let Some(encoding) = self.encoding {
                encoder = encoder.with_encoding(encoding);
            }

            let mut decoder = Decoder::new();

            if let Some(decoding) = self.decoding {
                decoder = decoder.with_opts(decoding);
            }

            Tester {
                generator,
                splitter,
                extractor,
                merger,
                encoder,
                decoder,
            }
        }
    }

    struct Tester {
        generator: PayloadGenerator,
        splitter: PayloadSplitter,
        extractor: PayloadExtractor,
        merger: PayloadMerger,
        encoder: Encoder,
        decoder: Decoder,
    }

    impl Tester {
        fn merge_one_by_one(
            &mut self,
            splits: impl IntoIterator<Item = impl Into<Payload>>,
        ) -> CompletePayload {
            let mut merger = self.merger.clone();

            let mut completes = Vec::new();
            for payload in splits {
                let res = merger.merge([payload]);
                completes.extend(res.complete);

                merger = PayloadMerger::default().with_unmerged(res.incomplete);
            }

            assert_eq!(completes.len(), 1);

            completes
                .into_iter()
                .next()
                .expect("should have exactly one complete payload")
        }

        fn test(mut self, data: Bytes) -> Result<(), RoundtripError> {
            let payload = self.generator.generate(data.clone())?;

            let splits = self.splitter.split(payload);

            let encoded = self.encoder.encode(splits)?;

            let decoded = encoded
                .into_iter()
                .try_fold(Vec::new(), |mut acc, encoded| {
                    let payloads = self.decoder.decode(encoded.as_bytes())?;
                    acc.extend(payloads);
                    Ok::<_, DecodingError>(acc)
                })?;

            let complete = self.merge_one_by_one(decoded);

            let extracted = self.extractor.extract(complete)?;

            assert_eq!(extracted, data);

            Ok(())
        }
    }

    #[test]
    fn test_no_encryption() {
        TesterBuilder::default()
            .build()
            .test("hello world".into())
            .expect("roundtrip failed");
    }

    #[test]
    fn test_no_encryption_splits() {
        TesterBuilder::default()
            .with_splits(Some(4))
            .build()
            .test("hello world".into())
            .expect("roundtrip failed");
    }

    #[test]
    fn test_json_no_merge() {
        TesterBuilder::default()
            .with_encoding(Some(EncodingOpts::Json {
                pretty: true,
                merge: false,
            }))
            .with_decoding(Some(DecodingOpts::Json))
            .build()
            .test("hello world".into())
            .expect("roundtrip failed");
    }

    #[test]
    fn spec_mismatch() {
        let err = TesterBuilder::default()
            .with_compression(Some(Compression::Gzip(GzipCompression)))
            .with_decompression(Some(Decompression::NoCompression))
            .build()
            .test("data".into())
            .err()
            .expect("should have failed");

        match err {
            RoundtripError::ExtractionError(PayloadExtractionError::DecompressionError(
                DecompressionError::SpecMismtach { payload, tried },
            )) => {
                assert!(matches!(payload, CompressionSpec::Gzip));
                assert!(matches!(tried, Decompression::NoCompression));
            }
            _ => panic!("should have failed with decompression error"),
        };
    }

    #[test]
    fn test_no_compression_splits() {
        TesterBuilder::default()
            .with_splits(Some(4))
            .with_compression(Some(Compression::Gzip(GzipCompression)))
            .with_decompression(Some(Decompression::Gzip(GzipCompression)))
            .build()
            .test("hello world".into())
            .expect("roundtrip failed");
    }

    #[test]
    fn test_age_passphrase_encryption() {
        let passphrase = AgePassphrase::new(SecretString::new("passphrase".into()));

        TesterBuilder::default()
            .with_encryption(Some(Encryption::AgePassphrase(passphrase.clone())))
            .with_decryption(Some(Decryption::AgePassphrase(passphrase)))
            .build()
            .test("hello world".into())
            .expect("roundtrip failed");
    }

    #[test]
    fn test_age_key_encryption() {
        let id0 = x25519::Identity::generate();
        let id1 = x25519::Identity::generate();
        let id2 = x25519::Identity::generate();

        let encryption =
            AgeKeyEncryption::new(vec![id0.to_public(), id1.to_public(), id2.to_public()]);
        let decryption = AgeKeyDecryption::new(vec![id2]);

        TesterBuilder::default()
            .with_encryption(Some(Encryption::AgeKey(encryption)))
            .with_decryption(Some(Decryption::AgeKey(decryption)))
            .build()
            .test("hello world".into())
            .expect("roundtrip failed");
    }

    #[test]
    fn test_age_key_no_matching() {
        let id = x25519::Identity::generate();

        let encryption = AgeKeyEncryption::new(vec![id.to_public()]);
        let decryption = AgeKeyDecryption::new(vec![]);

        let err = TesterBuilder::default()
            .with_encryption(Some(Encryption::AgeKey(encryption)))
            .with_decryption(Some(Decryption::AgeKey(decryption)))
            .build()
            .test("hello world".into())
            .err()
            .expect("should have failed");

        match err {
            RoundtripError::ExtractionError(PayloadExtractionError::DecryptionError(
                DecryptionError::Age(e),
            )) => {
                assert!(matches!(e, DecryptError::NoMatchingKeys));
            }
            _ => panic!("should have failed with decryption error"),
        };
    }
}
