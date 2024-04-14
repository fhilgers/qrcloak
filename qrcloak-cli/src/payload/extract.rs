use std::str::FromStr;

use clap::Parser;

use miette::IntoDiagnostic;
use qrcloak_core::payload::format::CompletePayload;
use serde::de::Error;

use crate::{decryption::DecryptionArgs, input::Input, OneOrMany};

#[derive(Parser, Debug)]
pub struct PayloadExtractArgs {
    #[command(flatten)]
    input: Input<PayloadExtractText>,

    #[command(flatten)]
    decryption: DecryptionArgs,
}

#[derive(Clone, Debug)]
struct PayloadExtractText(CompletePayload);

impl FromStr for PayloadExtractText {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payload = serde_json::from_str::<OneOrMany<CompletePayload>>(s)?;

        match payload {
            OneOrMany::One(payload) => Ok(PayloadExtractText(payload)),
            OneOrMany::Many(payloads) => {
                if payloads.len() == 1 {
                    Ok(PayloadExtractText(payloads.into_iter().next().unwrap()))
                } else {
                    Err(serde_json::Error::custom(
                        "Input is a list of multiple payloads",
                    ))
                }
            }
        }
    }
}

impl PayloadExtractArgs {
    pub fn handle(self) -> miette::Result<String> {
        let payload = self.input.contents().into_diagnostic()?.0;

        self.decryption.decrypt(&payload)
    }
}
