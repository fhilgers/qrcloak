// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::io::Write;
use std::vec;

use clap::Parser;

use miette::IntoDiagnostic;
use qrcloak_core::format::Payload;
use qrcloak_core::payload::{Encoder, EncodingOpts, PayloadGenerator, PayloadSplitter};

use crate::encryption::EncryptionOptions;
use crate::input::Input;
use crate::FileOrStdout;

#[derive(Parser, Debug)]
pub struct PayloadGenerateArgs {
    #[arg(short, long, help = "Split payload into {} parts")]
    splits: Option<u32>,

    #[command(flatten)]
    encryption: EncryptionOptions,

    #[command(flatten)]
    input: Input<String>,

    #[arg(long)]
    pretty: bool,

    #[arg(default_value_t = FileOrStdout::Stdout)]
    output: FileOrStdout,
}

impl PayloadGenerateArgs {
    pub fn handle(self) -> miette::Result<()> {
        let input = self.input.contents().into_diagnostic()?;

        let payloads = PayloadGenerator::default()
            .with_encryption(self.encryption.0)
            .generate(input.into())
            .into_diagnostic()?;

        let payloads: Vec<Payload> = if let Some(splits) = self.splits {
            PayloadSplitter::default()
                .with_splits(splits)
                .split(payloads)
                .map(Payload::from)
                .collect()
        } else {
            vec![Payload::from(payloads)]
        };

        let encoded_payloads = Encoder::default()
            .with_encoding(EncodingOpts::Json {
                pretty: self.pretty,
                merge: true,
            })
            .encode(payloads)
            .into_diagnostic()?;

        let mut writer = self.output.try_get_writer().into_diagnostic()?;

        writeln!(writer, "{}", encoded_payloads[0]).into_diagnostic()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use qrcloak_core::payload::{Decoder, PayloadExtractor, PayloadMerger};

    use super::*;

    #[test]
    fn test_generate_payload() {
        let mut args = PayloadGenerateArgs::parse_from([
            "cmd",
            "--splits",
            "2",
            "--pretty",
            "--text",
            "hello world",
        ]);

        let output = FileOrStdout::new_testing();
        args.output = output.clone();

        args.handle().unwrap();

        let payloads = Decoder::default()
            .decode(&output.into_inner())
            .expect("should decode");

        let mut complete = PayloadMerger::default().merge(payloads).complete;

        assert_eq!(complete.len(), 1);
        let complete = complete.pop().expect("should have one complete");

        let extracted = PayloadExtractor::default()
            .extract(complete)
            .expect("should extract");

        assert_eq!(&*extracted, b"hello world");
    }
}
