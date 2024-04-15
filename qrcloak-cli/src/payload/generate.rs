use std::io::Write;

use clap::Parser;

use miette::IntoDiagnostic;
use qrcloak_core::payload::Encoder;
use qrcloak_core::payload::PayloadBuilder;

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

        let payloads = PayloadBuilder::default()
            .with_encryption(self.encryption.0)
            .with_splits(self.splits)
            .build(&input)
            .into_diagnostic()?;

        let encoded_payloads = Encoder::default()
            .with_encoding(qrcloak_core::payload::EncodingOpts::Json {
                pretty: self.pretty,
                merge: true,
            })
            .encode(&payloads)
            .into_diagnostic()?;

        let mut writer = self.output.try_get_writer().into_diagnostic()?;

        writeln!(writer, "{}", encoded_payloads.first()).into_diagnostic()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use qrcloak_core::payload::{format::CompletePayload, Decoder, PayloadExtractor};

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

        let complete = CompletePayload::try_from(payloads).expect("should merge into complete");

        let extracted = PayloadExtractor::default()
            .extract(&complete)
            .expect("should extract");

        assert_eq!(extracted, b"hello world");
    }
}
