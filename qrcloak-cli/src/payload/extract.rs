use std::str::FromStr;

use clap::Parser;
use std::io::Write;

use miette::IntoDiagnostic;
use qrcloak_core::payload::{format::CompletePayload, OneOrMore};
use serde::de::Error;

use crate::{decryption::DecryptionArgs, input::Input, FileOrStdout};

#[derive(Parser, Debug)]
pub struct PayloadExtractArgs {
    #[command(flatten)]
    input: Input<PayloadExtractText>,

    #[command(flatten)]
    decryption: DecryptionArgs,

    #[arg(default_value_t = FileOrStdout::Stdout)]
    output: FileOrStdout,
}

#[derive(Clone, Debug)]
struct PayloadExtractText(CompletePayload);

impl FromStr for PayloadExtractText {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payload = serde_json::from_str::<OneOrMore<CompletePayload>>(s)?;

        if !payload.is_one() {
            return Err(serde_json::Error::custom(
                "Input is a list of multiple payloads",
            ));
        }

        Ok(Self(payload.first().clone()))
    }
}

impl PayloadExtractArgs {
    pub fn handle(self) -> miette::Result<()> {
        let payload = self.input.contents().into_diagnostic()?.0;

        let data = self.decryption.decrypt(&payload)?;

        let mut writer = self.output.try_get_writer().into_diagnostic()?;

        writeln!(writer, "{}", data).into_diagnostic()?;

        Ok(())
    }
}
