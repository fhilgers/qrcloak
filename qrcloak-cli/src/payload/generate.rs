use clap::Parser;

use miette::IntoDiagnostic;
use qrcloak_core::payload::{format::Payload, PayloadBuilder};

use crate::encryption::EncryptionOptions;
use crate::input::Input;

#[derive(Parser, Debug)]
pub struct PayloadGenerateArgs {
    #[arg(short, long, help = "Split payload into {} parts")]
    splits: Option<u32>,

    #[command(flatten)]
    encryption: EncryptionOptions,

    #[command(flatten)]
    input: Input<String>,
}

impl PayloadGenerateArgs {
    pub fn handle(self) -> miette::Result<Vec<Payload>> {
        let input = self.input.contents().into_diagnostic()?.into_bytes();

        let encryption = self.encryption.0;

        let mut builder = PayloadBuilder::default();

        builder = if let Some(opts) = encryption {
            builder.with_encryption(opts)
        } else {
            builder
        };

        if let Some(splits) = self.splits {
            builder.build_partial(&input, splits).into_diagnostic()
        } else {
            builder
                .build_complete(&input)
                .into_diagnostic()
                .map(|p| vec![p])
        }
    }
}
