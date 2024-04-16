use std::path::PathBuf;

use clap::Parser;
use miette::IntoDiagnostic;
use qrcloak_core::{
    generate::Generator,
    payload::{OneOrMore, PayloadGenerator, PayloadSplitter},
};

use crate::{encryption::EncryptionOptions, input::Input};

#[derive(Parser, Debug)]
pub struct QrCodeGenerateArgs {
    #[command(flatten)]
    encryption: EncryptionOptions,

    #[command(flatten)]
    input: Input<String>,

    #[arg(required = true)]
    output: Vec<PathBuf>,
}

fn ensure_parent(path: &PathBuf) -> miette::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).into_diagnostic()?;
    }
    Ok(())
}

impl QrCodeGenerateArgs {
    pub fn handle(self) -> miette::Result<()> {
        let input = self.input.contents().into_diagnostic()?;

        let splits = if self.output.len() > 1 {
            Some(self.output.len() as u32)
        } else {
            None
        };

        let payloads = PayloadGenerator::default()
            .with_encryption(self.encryption.0)
            .generate(input.into())
            .into_diagnostic()?;

        let payloads = if let Some(splits) = splits {
            PayloadSplitter::default()
                .with_splits(splits)
                .split(payloads)
                .into_payloads()
        } else {
            OneOrMore::from(payloads).into_payloads()
        };

        let images = Generator::default().generate(&payloads).into_diagnostic()?;

        for (image, path) in images.into_iter().zip(self.output.into_iter()) {
            ensure_parent(&path)?;

            image.save(path).into_diagnostic()?;
        }

        Ok(())
    }
}
