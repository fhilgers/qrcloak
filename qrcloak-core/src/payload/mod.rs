use core::panic;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::{self, Write},
    ops::Range,
};

#[cfg(feature = "json")]
mod decoder;
#[cfg(feature = "json")]
mod encoder;
mod extractor;
mod one_or_more;

use bytes::Bytes;
#[cfg(feature = "json")]
pub use decoder::{Decoder, DecodingError, DecodingOpts};

#[cfg(feature = "json")]
pub use encoder::{Encoder, EncodingError, EncodingOpts};
pub use extractor::{DecryptionOpts, EncryptionMismatch, ExtractionError, PayloadExtractor};
pub use one_or_more::OneOrMore;

pub mod format;

use age::{secrecy::SecretString, x25519, Recipient};

use thiserror::Error;

use self::format::{
    AgeKeySpec, AgePasswordSpec, CompletePayload, EncryptionSpec, IndexMetadata, PartialPayload,
    PartialPayloadData, Payload, PayloadMetadata,
};

#[derive(Debug, Error)]
pub struct ConversionError {
    pub data: Vec<PartialPayload>,

    pub source: ValidationError,
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("chain is empty")]
    Empty,

    #[error("chain is missing its head")]
    MissingHead,

    #[error("invalid size (expected {expected}, actual {actual})")]
    InvalidSize { expected: u32, actual: u32 },

    #[error("invalid size (expected {expected:?}, actual {actual})")]
    InvalidIndex { expected: Range<u32>, actual: u32 },

    #[error("found multiple ids: {ids:?}")]
    MultipleIds { ids: Vec<u32> },

    #[error("found multiple sizes: {sizes:?}")]
    MultipleSizes { sizes: Vec<u32> },

    #[error("duplicate indeces: {indeces:?}")]
    DuplicateIndeces { indeces: BTreeSet<u32> },

    #[error("duplicate indeces: {indeces:?}")]
    MissingIndeces { indeces: BTreeSet<u32> },
}

pub fn extract_chains(
    values: impl Into<Vec<Payload>>,
) -> (Vec<CompletePayload>, Vec<PartialPayload>) {
    let (mut complete, partial) = values.into().into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut complete, mut partial), value| {
            match value {
                Payload::Complete(c) => complete.push(c),
                Payload::Partial(p) => partial.push(p),
            };
            (complete, partial)
        },
    );

    let mut groups = BTreeMap::<u32, Vec<PartialPayload>>::new();

    for value in partial {
        let id = value.index_metadata.id;

        let entry = groups.entry(id).or_default();
        entry.push(value);
    }
    let mut invalid = vec![];

    for group in groups.into_values() {
        match convert_chain(group) {
            Ok(c) => complete.push(c),
            Err(e) => invalid.extend(e.data),
        }
    }

    (complete, invalid)
}

pub fn convert_chain(
    value: impl Into<Vec<PartialPayload>>,
) -> Result<CompletePayload, ConversionError> {
    let value = value.into();
    if let Err(err) = validate_chain(&value) {
        return Err(ConversionError {
            data: value,
            source: err,
        });
    }

    let mut metadata = None;
    let mut data = vec![None; value.len()];

    for p in value {
        let index = p.index_metadata.index as usize;

        if index == 0 {
            if let PartialPayloadData::Head(head) = p.data {
                metadata = Some(head.payload_metadata);
                data[index] = Some(head.data)
            } else {
                panic!("checked pefore");
            }
        } else {
            if let PartialPayloadData::Tail { data: tail } = p.data {
                data[index] = Some(tail)
            } else {
                panic!("checked pefore");
            }
        }
    }

    Ok(CompletePayload {
        payload_metadata: metadata.expect("must be there"),
        data: data
            .into_iter()
            .flat_map(|maybe_data| maybe_data.expect("must be there"))
            .collect(),
    })
}

pub fn validate_chain(value: &[PartialPayload]) -> Result<(), ValidationError> {
    if value.len() == 0 {
        return Err(ValidationError::Empty);
    }

    if value.len() == 1 {
        let head = &value[0];

        if head.index_metadata.size != 1 {
            return Err(ValidationError::InvalidSize {
                expected: 1,
                actual: head.index_metadata.size,
            });
        }

        if head.index_metadata.index != 0 {
            return Err(ValidationError::InvalidIndex {
                expected: 0..0,
                actual: head.index_metadata.index,
            });
        }

        if !matches!(head.data, PartialPayloadData::Head(_)) {
            return Err(ValidationError::MissingHead);
        }

        return Ok(());
    }

    let first_size = value[0].index_metadata.size;

    let mut ids = BTreeSet::new();
    let mut sizes = BTreeSet::new();

    let mut payloads = BTreeMap::<u32, Vec<&PartialPayload>>::new();

    if first_size as usize != value.len() {
        return Err(ValidationError::InvalidSize {
            expected: value.len() as u32,
            actual: first_size,
        });
    }

    for p in value.iter() {
        let IndexMetadata { id, index, size } = p.index_metadata;

        ids.insert(id);
        sizes.insert(size);

        match &p.data {
            PartialPayloadData::Head(_) if index > 0 => {
                return Err(ValidationError::InvalidIndex {
                    expected: 0..0,
                    actual: index,
                })
            }
            PartialPayloadData::Tail { .. } if index == 0 => {
                return Err(ValidationError::InvalidIndex {
                    expected: 1..size,
                    actual: index,
                })
            }
            _ => {
                let entry = payloads.entry(index).or_insert(Vec::new());

                if !entry.contains(&p) {
                    entry.push(p);
                }
            }
        }
    }

    if ids.len() > 1 {
        return Err(ValidationError::MultipleIds {
            ids: ids.into_iter().collect(),
        });
    }

    if sizes.len() > 1 {
        return Err(ValidationError::MultipleSizes {
            sizes: sizes.into_iter().collect(),
        });
    }

    if payloads.len() != first_size as usize {
        let indeces = (0..first_size)
            .filter(|index| !payloads.contains_key(index))
            .collect();
        return Err(ValidationError::MissingIndeces { indeces });
    }

    payloads.retain(|_, value| value.len() > 1);

    if payloads.len() != 0 {
        let indeces = payloads.keys().cloned().collect();
        return Err(ValidationError::DuplicateIndeces { indeces });
    }

    Ok(())
}

impl TryFrom<Vec<PartialPayload>> for CompletePayload {
    type Error = ConversionError;

    fn try_from(value: Vec<PartialPayload>) -> Result<Self, Self::Error> {
        convert_chain(value)
    }
}

impl TryFrom<PartialPayload> for CompletePayload {
    type Error = ConversionError;

    fn try_from(value: PartialPayload) -> Result<Self, Self::Error> {
        convert_chain(vec![value])
    }
}

impl CompletePayload {
    pub fn split(self, num_parts: u32) -> Vec<PartialPayload> {
        if num_parts == 0 {
            panic!("num_parts has to be > 0")
        }
        if num_parts as usize > self.data.len() {
            panic!(
                "num_parts ({num_parts}) has to be <= self.data.len ({})",
                self.data.len()
            )
        }

        let len = self.data.len();
        let (quo, rem) = (len / num_parts as usize, len % num_parts as usize);
        let quo1 = quo + 1;

        let id = rand::random();

        let mut chunked_data = Vec::with_capacity(num_parts as usize);

        let make_head = |data: Bytes, index: u32| PartialPayload {
            index_metadata: IndexMetadata {
                id,
                index,
                size: num_parts,
            },
            data: PartialPayloadData::Head(CompletePayload {
                payload_metadata: self.payload_metadata.clone(),
                data,
            }),
        };

        let make_tail = |data: Bytes, index: u32| PartialPayload {
            index_metadata: IndexMetadata {
                id,
                index: index as u32,
                size: num_parts,
            },
            data: PartialPayloadData::Tail { data },
        };

        let make_partial = |data: Bytes, index: u32| {
            if index == 0 {
                make_head(data, index)
            } else {
                make_tail(data, index)
            }
        };

        for i in 0..rem {
            let left = i * quo1;
            chunked_data.push(make_partial(self.data.slice(left..left + quo1), i as u32));
        }
        for i in rem..num_parts as usize {
            let left = rem + i * (quo);
            chunked_data.push(make_partial(self.data.slice(left..left + quo), i as u32));
        }

        chunked_data
    }
}

#[derive(Debug, Clone)]
pub enum EncryptionOptions {
    AgeKey(AgeKeyOptions),
    AgePassword(AgePasswordOptions),
}

#[derive(Debug, Clone)]
pub struct AgeKeyOptions {
    recipients: Vec<x25519::Recipient>,
}

impl AgeKeyOptions {
    pub fn new(recipients: &[x25519::Recipient]) -> Self {
        AgeKeyOptions {
            recipients: recipients.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("missing passphrase or recipient")]
    MissingKey,

    #[error("failed to encrypt")]
    EncryptError(#[from] age::EncryptError),

    #[error(transparent)]
    IoError(#[from] io::Error),
}

impl AgeKeyOptions {
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let recipients = self
            .recipients
            .iter()
            .cloned()
            .map(|key| Box::new(key) as Box<dyn Recipient + Send>)
            .collect();

        let encryptor =
            age::Encryptor::with_recipients(recipients).ok_or(EncryptionError::MissingKey)?;

        let mut encrypted_data = Vec::with_capacity(data.len());

        let mut writer = encryptor.wrap_output(&mut encrypted_data)?;
        writer.write_all(&data)?;
        writer.finish()?;

        Ok(encrypted_data)
    }

    pub fn to_spec(&self) -> EncryptionSpec {
        EncryptionSpec::AgeKey(AgeKeySpec)
    }
}

#[derive(Debug, Clone)]
pub struct AgePasswordOptions {
    passphrase: SecretString,
}

impl AgePasswordOptions {
    pub fn new(password: &str) -> Self {
        AgePasswordOptions {
            passphrase: SecretString::new(password.into()),
        }
    }
}

impl AgePasswordOptions {
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let encryptor = age::Encryptor::with_user_passphrase(self.passphrase.clone());

        let mut encrypted_data = Vec::with_capacity(data.len());

        let mut writer = encryptor.wrap_output(&mut encrypted_data)?;
        writer.write_all(&data)?;
        writer.finish()?;

        Ok(encrypted_data)
    }

    pub fn to_spec(&self) -> EncryptionSpec {
        EncryptionSpec::AgePassword(AgePasswordSpec)
    }
}

#[derive(Default)]
pub struct PayloadBuilder {
    encryption: Option<EncryptionOptions>,
    splits: Option<u32>,
}

impl PayloadBuilder {
    pub fn with_encryption(mut self, encryption: Option<EncryptionOptions>) -> Self {
        self.encryption = encryption;
        self
    }

    pub fn with_splits(mut self, splits: Option<u32>) -> Self {
        self.splits = splits;
        self
    }

    fn build_complete_inner(self, data: &[u8]) -> Result<CompletePayload, EncryptionError> {
        let (spec, data) = match self.encryption {
            Some(EncryptionOptions::AgeKey(key)) => (Some(key.to_spec()), key.encrypt(data)?),
            Some(EncryptionOptions::AgePassword(pw)) => (Some(pw.to_spec()), pw.encrypt(data)?),
            None => (None, data.to_vec()),
        };

        Ok(CompletePayload {
            payload_metadata: PayloadMetadata {
                encryption: spec,
                compression: None,
            },
            data: data.into(),
        })
    }

    pub fn build(self, data: &str) -> Result<OneOrMore<'static, Payload>, EncryptionError> {
        if let Some(splits) = self.splits {
            self.build_partial(data.as_bytes(), splits)
                .map(|v| OneOrMore::try_from(v).expect("at least one element"))
        } else {
            self.build_complete(data.as_bytes()).map(OneOrMore::from)
        }
    }

    fn build_complete(self, data: &[u8]) -> Result<Payload, EncryptionError> {
        Ok(Payload::Complete(self.build_complete_inner(data)?))
    }

    fn build_partial(self, data: &[u8], parts: u32) -> Result<Vec<Payload>, EncryptionError> {
        Ok(self
            .build_complete_inner(data)?
            .split(parts)
            .into_iter()
            .map(|partial| Payload::Partial(partial))
            .collect())
    }
}

#[cfg(test)]
mod test {
    use std::str::from_utf8;

    use age::secrecy::SecretString;
    use rand::{seq::SliceRandom, thread_rng};

    use crate::payload::CompletePayload;

    use super::PayloadBuilder;

    const DATA: &'static str = "hello world";

    #[test]
    fn payload_complete_no_encryption() {
        let payload = PayloadBuilder::default()
            .build_complete(DATA.as_bytes())
            .expect("should not fail");

        match payload {
            crate::payload::Payload::Complete(c) => {
                assert!(c.payload_metadata.encryption.is_none());
                assert!(c.payload_metadata.compression.is_none());
                assert_eq!(c.data.len(), DATA.len());
            }
            _ => panic!("payload should be complete"),
        }
    }

    #[test]
    fn payload_partial_no_encryption() {
        let payload = PayloadBuilder::default()
            .build_partial(DATA.as_bytes(), DATA.len() as u32)
            .expect("should not fail");

        assert_eq!(payload.len(), DATA.len());

        let mut partials = payload
            .into_iter()
            .map(|payload| match payload {
                crate::payload::Payload::Complete(_) => panic!("expected partials"),
                crate::payload::Payload::Partial(p) => p,
            })
            .collect::<Vec<_>>();

        partials.as_mut_slice().shuffle(&mut thread_rng());

        let complete: CompletePayload = partials.try_into().expect("should merge");

        let data = from_utf8(&complete.data).expect("should be utf8");

        assert_eq!(data, DATA);
    }

    #[test]
    fn payload_complete_age_passphrase() {
        let _payload = PayloadBuilder::default()
            .with_encryption(Some(super::EncryptionOptions::AgePassword(
                super::AgePasswordOptions {
                    passphrase: SecretString::new("hello".into()),
                },
            )))
            .build_complete(DATA.as_bytes())
            .expect("encryption should not fail");
    }
}
