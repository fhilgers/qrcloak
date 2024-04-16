#[cfg(feature = "json")]
mod decoder;

#[cfg(feature = "json")]
mod encoder;

mod extract;
mod generate;
mod merge;
mod one_or_more;
mod split;
mod utils;

#[cfg(feature = "json")]
pub use decoder::{Decoder, DecodingError, DecodingOpts};
#[cfg(feature = "json")]
pub use encoder::{Encoder, EncodingError, EncodingOpts};

pub use extract::{PayloadExtractionError, PayloadExtractor};
pub use generate::{PayloadGenerationError, PayloadGenerator};
pub use merge::{MergeResult, PayloadMerger};
pub use split::PayloadSplitter;

#[cfg(all(test, feature = "json"))]
mod tests {

    use core::panic;

    use bytes::Bytes;

    use crate::{
        format::{AgeKeyDecryption, AgeKeyEncryption, Payload},
        payload_new::{
            extract::PayloadExtractor,
            merge::{MergeResult, PayloadMerger},
            Decoder, Encoder,
        },
    };

    use super::{generate::PayloadGenerator, split::PayloadSplitter};

    #[test]
    fn test_roundtrip() {
        let expected = b"hello world";

        let payload = PayloadGenerator::default()
            .generate(Bytes::from_static(expected))
            .expect("should generate");

        let split = PayloadSplitter::default()
            .with_splits(4)
            .split(payload)
            .into_payloads();

        assert_eq!(split.len(), 4);

        let encoded = Encoder::default()
            .with_encoding(crate::payload_new::EncodingOpts::Json {
                pretty: true,
                merge: true,
            })
            .encode(&split)
            .expect("should encode");

        let (complete, partial) = Decoder::default()
            .with_opts(crate::payload_new::DecodingOpts::Json)
            .decode(encoded[0].as_bytes())
            .expect("should decode")
            .split();

        assert_eq!(complete.len(), 0);
        assert_eq!(partial.len(), 4);

        let MergeResult(mut complete, _) = PayloadMerger::default().merge(partial);

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
            .with_encryption(crate::format::Encryption::AgeKey(AgeKeyEncryption::new(
                vec![identity.to_public()],
            )))
            .generate(Bytes::from_static(expected))
            .expect("should generate");

        let split = PayloadSplitter::default().with_splits(4).split(payload);

        assert_eq!(split.len(), 4);

        let MergeResult(mut complete, _) = PayloadMerger::default().merge(split.into());

        assert_eq!(complete.len(), 1);

        let complete = complete.pop().expect("should have one complete");

        let data = PayloadExtractor::default()
            .with_decryption(crate::format::Decryption::AgeKey(AgeKeyDecryption::new(
                vec![identity],
            )))
            .extract(complete)
            .expect("should extract");

        assert_eq!(&*data, expected);
    }
}
