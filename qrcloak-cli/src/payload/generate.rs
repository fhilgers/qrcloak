use clap::Parser;

use miette::IntoDiagnostic;
use qrcloak_core::payload::{format::Payload, PayloadBuilder};
use qrcloak_core::payload::{Encoder, OneOrMore};

use crate::encryption::EncryptionOptions;
use crate::input::Input;

#[derive(Parser, Debug)]
pub struct PayloadGenerateArgs {
    #[arg(short, long, help = "Split payload into {} parts")]
    pub splits: Option<u32>,

    #[command(flatten)]
    encryption: EncryptionOptions,

    #[command(flatten)]
    input: Input<String>,
}

impl PayloadGenerateArgs {
    pub fn handle(self) -> miette::Result<OneOrMore<'static, Payload>> {
        let input = self.input.contents().into_diagnostic()?;

        let payloads = PayloadBuilder::default()
            .with_encryption(self.encryption.0)
            .with_splits(self.splits)
            .build(&input)
            .into_diagnostic()?;

        Ok(payloads)
    }

    pub fn handle_with_encoding(self) -> miette::Result<OneOrMore<'static, String>> {
        let payloads = self.handle()?;

        Encoder::default().encode(&payloads).into_diagnostic()
    }
}
