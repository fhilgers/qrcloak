use std::fmt::Debug;

use age::{secrecy::SecretString, x25519};
use clap::{Args, CommandFactory, FromArgMatches, Parser};
use qrcloak_core::payload::{AgeKeyDecryption, AgePassphrase, Decryption};

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

#[derive(Clone, Debug)]
pub struct DecryptionOptions(pub Decryption);

impl Parser for DecryptionOptions {}

impl CommandFactory for DecryptionOptions {
    fn command() -> clap::Command {
        DecryptionArgsInner::command()
    }
    fn command_for_update() -> clap::Command {
        DecryptionArgsInner::command_for_update()
    }
}

impl Args for DecryptionOptions {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        DecryptionArgsInner::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        DecryptionArgsInner::augment_args_for_update(cmd)
    }
}

impl TryFrom<DecryptionArgsInner> for DecryptionOptions {
    type Error = clap::Error;

    fn try_from(inner: DecryptionArgsInner) -> Result<DecryptionOptions, Self::Error> {
        if inner.age_key {
            let key: x25519::Identity = get_env("AGE_PRIVATE_KEY")?;

            Ok(DecryptionOptions(Decryption::AgeKey(
                AgeKeyDecryption::new(vec![key]),
            )))
        } else if inner.age_passphrase {
            let passphrase = get_env("AGE_PASSPHRASE")?;

            Ok(DecryptionOptions(Decryption::AgePassphrase(
                AgePassphrase::new(SecretString::new(passphrase)),
            )))
        } else {
            Ok(DecryptionOptions(Decryption::NoEncryption))
        }
    }
}

impl From<&DecryptionOptions> for DecryptionArgsInner {
    fn from(args: &DecryptionOptions) -> Self {
        match args.0 {
            Decryption::AgeKey(_) => DecryptionArgsInner {
                age_key: true,
                age_passphrase: false,
            },
            Decryption::AgePassphrase(_) => DecryptionArgsInner {
                age_key: false,
                age_passphrase: true,
            },
            Decryption::NoEncryption => DecryptionArgsInner {
                age_key: false,
                age_passphrase: false,
            },
        }
    }
}

impl FromArgMatches for DecryptionOptions {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let inner = DecryptionArgsInner::from_arg_matches(matches)?;

        Self::try_from(inner)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        let mut inner = DecryptionArgsInner::from(&*self);

        inner.update_from_arg_matches(matches)?;

        *self = Self::try_from(inner)?;

        Ok(())
    }
}
