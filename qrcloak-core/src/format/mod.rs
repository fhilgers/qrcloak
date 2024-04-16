mod base45;
mod complete;
mod compression;
mod encryption;
mod index;
mod partial;

pub enum Payload {
    Complete(CompletePayload),
    Partial(PartialPayload),
}

pub use complete::CompletePayload;
pub use compression::{Compression, CompressionError, Decompression, DecompressionError};
pub use encryption::{
    AgeKeyDecryption, AgeKeyEncryption, AgePassphrase, Decryption, DecryptionError, Encryption,
    EncryptionError,
};
pub use index::Index;
pub use partial::{PartialPayload, PartialPayloadHead, PartialPayloadTail};
