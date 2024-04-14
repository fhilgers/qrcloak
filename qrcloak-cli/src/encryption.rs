use std::str::FromStr;

use clap::{Args, FromArgMatches};

use qrcloak_core::payload::{AgeKeyOptions, AgePasswordOptions};

use crate::env::get_env;

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
pub struct EncryptionArgs {
    #[arg(
        long,
        action,
        help = "Read public keys from $AGE_KEY environment variable (comma-separated)"
    )]
    age_key: bool,

    #[arg(
        long,
        action,
        help = "Read passphrase from $AGE_PASSPHRASE environment variable"
    )]
    age_passphrase: bool,
}

#[derive(Debug, Clone)]
pub struct EncryptionOptions(pub Option<qrcloak_core::payload::EncryptionOptions>);

impl Args for EncryptionOptions {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        EncryptionArgs::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        EncryptionArgs::augment_args_for_update(cmd)
    }
}

struct Recipients(Vec<age::x25519::Recipient>);

impl FromStr for Recipients {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .split(',')
            .try_fold(Recipients(Vec::new()), |mut acc, s| {
                acc.0.push(age::x25519::Recipient::from_str(s)?);

                Ok(acc)
            })
    }
}

impl From<&EncryptionOptions> for EncryptionArgs {
    fn from(args: &EncryptionOptions) -> Self {
        match args.0 {
            Some(qrcloak_core::payload::EncryptionOptions::AgeKey(_)) => EncryptionArgs {
                age_key: true,
                age_passphrase: false,
            },
            Some(qrcloak_core::payload::EncryptionOptions::AgePassword(_)) => EncryptionArgs {
                age_key: false,
                age_passphrase: true,
            },
            None => EncryptionArgs {
                age_key: false,
                age_passphrase: false,
            },
        }
    }
}

impl TryFrom<EncryptionArgs> for EncryptionOptions {
    type Error = clap::Error;

    fn try_from(args: EncryptionArgs) -> Result<Self, Self::Error> {
        if args.age_key {
            let reciptiens: Recipients = get_env("AGE_KEY")?;

            Ok(EncryptionOptions(Some(
                qrcloak_core::payload::EncryptionOptions::AgeKey(AgeKeyOptions::new(&reciptiens.0)),
            )))
        } else if args.age_passphrase {
            let passphrase: String = get_env("AGE_PASSPHRASE")?;

            Ok(EncryptionOptions(Some(
                qrcloak_core::payload::EncryptionOptions::AgePassword(AgePasswordOptions::new(
                    &passphrase,
                )),
            )))
        } else {
            Ok(EncryptionOptions(None))
        }
    }
}

impl FromArgMatches for EncryptionOptions {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let inner = EncryptionArgs::from_arg_matches(matches)?;

        inner.try_into()
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        let mut inner: EncryptionArgs = (&*self).into();

        inner.update_from_arg_matches(matches)?;

        *self = inner.try_into()?;

        Ok(())
    }
}