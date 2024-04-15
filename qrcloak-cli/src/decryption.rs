use std::fmt::Debug;

use age::{
    secrecy::{ExposeSecret, SecretString},
    x25519,
};
use clap::{Args, CommandFactory, FromArgMatches, Parser};
use miette::{miette, IntoDiagnostic};
use qrcloak_core::payload::format::{CompletePayload, EncryptionSpec};

use crate::env::get_env;

#[derive(Parser, Debug)]
struct DecryptionArgsInner {
    #[arg(
        long,
        help = "Read private key from $AGE_PRIVATE_KEY environment variable",
        group = "decryption"
    )]
    age_key: bool,

    #[arg(
        long,
        help = "Read passphrase from $AGE_PASSPHRASE environment variable",
        group = "decryption"
    )]
    age_passphrase: bool,
}

#[derive(Clone)]
pub enum DecryptionArgs {
    NoEncryption,
    AgeKey(x25519::Identity),
    AgePassphrase(SecretString),
}

impl Debug for DecryptionArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecryptionArgs::NoEncryption => write!(f, "NoEncryption"),
            DecryptionArgs::AgeKey(key) => write!(f, "AgeKey({:?})", key.to_string()),
            DecryptionArgs::AgePassphrase(passphrase) => write!(f, "AgePassphrase({passphrase:?})"),
        }
    }
}

impl Parser for DecryptionArgs {}

impl CommandFactory for DecryptionArgs {
    fn command() -> clap::Command {
        DecryptionArgsInner::command()
    }
    fn command_for_update() -> clap::Command {
        DecryptionArgsInner::command_for_update()
    }
}

impl Args for DecryptionArgs {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        DecryptionArgsInner::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        DecryptionArgsInner::augment_args_for_update(cmd)
    }
}

impl TryFrom<DecryptionArgsInner> for DecryptionArgs {
    type Error = clap::Error;

    fn try_from(inner: DecryptionArgsInner) -> Result<DecryptionArgs, Self::Error> {
        if inner.age_key {
            let key: x25519::Identity = get_env("AGE_PRIVATE_KEY")?;

            Ok(DecryptionArgs::AgeKey(key))
        } else if inner.age_passphrase {
            let passphrase = get_env("AGE_PASSPHRASE")?;

            Ok(DecryptionArgs::AgePassphrase(SecretString::new(passphrase)))
        } else {
            Ok(DecryptionArgs::NoEncryption)
        }
    }
}

impl FromArgMatches for DecryptionArgs {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let inner = DecryptionArgsInner::from_arg_matches(matches)?;

        Self::try_from(inner)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        let mut inner = match &self {
            DecryptionArgs::AgeKey(_) => DecryptionArgsInner {
                age_key: true,
                age_passphrase: false,
            },
            DecryptionArgs::AgePassphrase(_) => DecryptionArgsInner {
                age_key: false,
                age_passphrase: true,
            },
            DecryptionArgs::NoEncryption => DecryptionArgsInner {
                age_key: false,
                age_passphrase: false,
            },
        };

        inner.update_from_arg_matches(matches)?;

        *self = Self::try_from(inner)?;

        Ok(())
    }
}

impl DecryptionArgs {
    pub fn decrypt(&self, payload: &CompletePayload) -> miette::Result<String> {
        match (self, &payload.payload_metadata.encryption) {
            (DecryptionArgs::AgeKey(key), Some(EncryptionSpec::AgeKey(spec))) => {
                String::from_utf8(spec.decrypt(&key, &payload.data).into_diagnostic()?)
                    .into_diagnostic()
            }
            (
                DecryptionArgs::AgePassphrase(passphrase),
                Some(EncryptionSpec::AgePassword(spec)),
            ) => String::from_utf8(
                spec.decrypt(passphrase.expose_secret(), &payload.data)
                    .into_diagnostic()?,
            )
            .into_diagnostic(),
            (DecryptionArgs::NoEncryption, None) => {
                String::from_utf8(payload.data.to_vec()).into_diagnostic()
            }
            (have, got) => Err(miette!(
                "Payload is encrypted with {got:?} but {have:?} was provided"
            )),
        }
    }
}
