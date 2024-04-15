use std::str::FromStr;

use clap::Parser;

use miette::{Context, IntoDiagnostic};
use qrcloak_core::payload::{
    format::{PartialPayload, Payload},
    Encoder, OneOrMore,
};

use std::io::Write;

use crate::{input::Input, FileOrStdout};

#[derive(Parser, Debug)]
pub struct PayloadMergeArgs {
    #[command(flatten)]
    input: Input<PartialPayloads>,

    #[arg(long)]
    pretty: bool,

    #[arg(default_value_t = FileOrStdout::Stdout)]
    output: FileOrStdout,
}

impl PayloadMergeArgs {
    pub fn handle(self) -> miette::Result<()> {
        let payloads = self
            .input
            .contents()
            .into_diagnostic()
            .wrap_err("Unable to read payloads from input")?;

        let complete_payload =
            qrcloak_core::payload::convert_chain(payloads.0).into_diagnostic()?;

        let encoded_payload = Encoder::default()
            .with_encoding(qrcloak_core::payload::EncodingOpts::Json {
                pretty: self.pretty,
                merge: true,
            })
            .encode(&OneOrMore::from(Payload::from(complete_payload)))
            .into_diagnostic()?;

        let mut writer = self.output.try_get_writer().into_diagnostic()?;

        writeln!(writer, "{}", encoded_payload.first()).into_diagnostic()?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct PartialPayloads(Vec<PartialPayload>);

impl FromStr for PartialPayloads {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payloads = serde_json::from_str::<OneOrMore<PartialPayload>>(s)?;

        Ok(PartialPayloads(payloads.as_slice().to_vec()))
    }
}
