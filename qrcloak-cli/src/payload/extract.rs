use std::str::FromStr;

use clap::Parser;
use qrcloak_core::{
    format::CompletePayload,
    payload::{OneOrMany, PayloadExtractor},
};
use std::io::Write;

use miette::IntoDiagnostic;
use serde::de::Error;

use crate::{decryption::DecryptionOptions, input::Input, FileOrStdout};

#[derive(Parser, Debug)]
pub struct PayloadExtractArgs {
    #[command(flatten)]
    input: Input<PayloadExtractText>,

    #[command(flatten)]
    decryption: DecryptionOptions,

    #[arg(default_value_t = FileOrStdout::Stdout)]
    output: FileOrStdout,
}

#[derive(Clone, Debug)]
struct PayloadExtractText(CompletePayload);

impl FromStr for PayloadExtractText {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payload = serde_json::from_str::<OneOrMany<CompletePayload>>(s)?;

        match payload {
            OneOrMany::One(payload) => Ok(Self(payload)),
            OneOrMany::Many(mut payloads) => {
                if payloads.len() != 1 {
                    return Err(serde_json::Error::custom(
                        "Input is a list of multiple payloads",
                    ));
                }
                Ok(Self(payloads.pop().unwrap()))
            }
            OneOrMany::Empty => Err(serde_json::Error::custom("Input is empty")),
        }
    }
}

impl PayloadExtractArgs {
    pub fn handle(self) -> miette::Result<()> {
        let payload = self.input.contents().into_diagnostic()?.0;

        let data = PayloadExtractor::default()
            .with_decryption(self.decryption.0)
            .extract(payload)
            .into_diagnostic()?;

        let mut writer = self.output.try_get_writer().into_diagnostic()?;

        writeln!(writer, "{}", String::from_utf8_lossy(&data)).into_diagnostic()?;

        Ok(())
    }
}
