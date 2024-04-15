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
    pub splits: Option<u32>,

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
