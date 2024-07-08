// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    fmt::Debug,
    io::{self, Write},
};

use age::{secrecy::SecretString, x25519, DecryptError, EncryptError, Identity, Recipient};
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct AgeKeyEncryption {
    recipients: Vec<x25519::Recipient>,
}

impl From<Vec<x25519::Recipient>> for AgeKeyEncryption {
    fn from(recipients: Vec<x25519::Recipient>) -> Self {
        Self { recipients }
    }
}

impl Into<Vec<x25519::Recipient>> for AgeKeyEncryption {
    fn into(self) -> Vec<x25519::Recipient> {
        self.recipients
    }
}

impl AgeKeyEncryption {
    // TODO one or more
    pub fn new(recipients: Vec<x25519::Recipient>) -> Self {
        assert!(recipients.len() >= 1);

        Self { recipients }
    }

    pub fn encrypt(&self, data: Bytes) -> Result<Bytes, EncryptError> {
        let recipients = self
            .recipients
            .iter()
            .cloned()
            .map(|key| Box::new(key) as Box<dyn Recipient + Send>)
            .collect();
        let encryptor =
            age::Encryptor::with_recipients(recipients).expect("recipients should be more than 1");

        let mut writer = BytesMut::with_capacity(data.len()).writer();

        let mut crypt_writer = encryptor.wrap_output(&mut writer)?;
        crypt_writer.write_all(&data)?;
        crypt_writer.finish()?;

        Ok(writer.into_inner().freeze())
    }
}

#[derive(Clone)]
pub struct AgeKeyDecryption {
    identities: Vec<x25519::Identity>,
}

impl From<Vec<x25519::Identity>> for AgeKeyDecryption {
    fn from(identities: Vec<x25519::Identity>) -> Self {
        Self { identities }
    }
}

impl Into<Vec<x25519::Identity>> for AgeKeyDecryption {
    fn into(self) -> Vec<x25519::Identity> {
        self.identities
    }
}

impl Debug for AgeKeyDecryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgeKeyDecryption")
            .field(
                "identities",
                &self
                    .identities
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl AgeKeyDecryption {
    // TODO one or more
    pub fn new(identities: Vec<x25519::Identity>) -> Self {
        Self { identities }
    }

    pub fn decrypt(&self, data: Bytes) -> Result<Bytes, DecryptError> {
        let identities = self.identities.iter().map(|key| key as &dyn Identity);

        let len = data.len();
        let decryptor = match age::Decryptor::new(data.reader())? {
            age::Decryptor::Recipients(c) => Ok(c),
            age::Decryptor::Passphrase(_) => Err(DecryptError::NoMatchingKeys),
        }?;

        let mut writer = BytesMut::with_capacity(len).writer();

        let mut reader = decryptor.decrypt(identities)?;
        io::copy(&mut reader, &mut writer)?;

        Ok(writer.into_inner().freeze())
    }
}

#[derive(Debug, Clone)]
pub struct AgePassphrase {
    passphrase: SecretString,
}

impl From<SecretString> for AgePassphrase {
    fn from(passphrase: SecretString) -> Self {
        Self { passphrase }
    }
}

impl Into<SecretString> for AgePassphrase {
    fn into(self) -> SecretString {
        self.passphrase
    }
}

impl AgePassphrase {
    pub fn new(passphrase: SecretString) -> Self {
        Self::from(passphrase)
    }

    // TODO better error handling
    pub fn decrypt(&self, data: Bytes) -> Result<Bytes, DecryptError> {
        let len = data.len();
        let decryptor = match age::Decryptor::new(data.reader())? {
            age::Decryptor::Recipients(_) => Err(DecryptError::NoMatchingKeys),
            age::Decryptor::Passphrase(c) => Ok(c),
        }?;

        let mut writer = BytesMut::with_capacity(len).writer();

        let mut reader = decryptor.decrypt(&self.passphrase, None)?;
        io::copy(&mut reader, &mut writer)?;

        Ok(writer.into_inner().freeze())
    }

    pub fn encrypt(&self, data: Bytes) -> Result<Bytes, EncryptError> {
        let encryptor = age::Encryptor::with_user_passphrase(self.passphrase.clone());

        let mut writer = BytesMut::with_capacity(data.len()).writer();

        let mut crypt_writer = encryptor.wrap_output(&mut writer)?;
        crypt_writer.write_all(&data)?;
        crypt_writer.finish()?;

        Ok(writer.into_inner().freeze())
    }
}
