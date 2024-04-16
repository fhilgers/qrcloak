mod extract;
mod generate;
mod merge;
mod split;
mod utils;

#[cfg(test)]
mod tests {

    use bytes::Bytes;

    use crate::{
        format::{AgeKeyDecryption, AgeKeyEncryption},
        payload_new::{
            extract::PayloadExtractor,
            merge::{MergeResult, PayloadMerger},
        },
    };

    use super::{generate::PayloadGenerator, split::PayloadSplitter};

    #[test]
    fn test_roundtrip() {
        let expected = b"hello world";

        let payload = PayloadGenerator::default()
            .generate(Bytes::from_static(expected))
            .expect("should generate");

        let split = PayloadSplitter::default().with_splits(4).split(payload);

        assert_eq!(split.len(), 4);

        let MergeResult(mut complete, _) = PayloadMerger::default().merge(split);

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

        let MergeResult(mut complete, _) = PayloadMerger::default().merge(split);

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
