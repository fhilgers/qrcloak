use std::str::FromStr;

use clap::Parser;

use miette::{miette, Context, IntoDiagnostic};
use qrcloak_core::{
    format::PartialPayload,
    payload::{Encoder, EncodingOpts, OneOrMany, PayloadMerger},
};
use serde::de::Error;

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

        let merge_result = PayloadMerger::default().merge(payloads.0);

        // TODO: create good error message
        if merge_result.0.len() != 1 {
            return Err(miette!("Merge result should have one complete payload"));
        }

        let encoded_payload = Encoder::default()
            .with_encoding(EncodingOpts::Json {
                pretty: self.pretty,
                merge: true,
            })
            .encode(merge_result.0)
            .into_diagnostic()?;

        let mut writer = self.output.try_get_writer().into_diagnostic()?;

        writeln!(writer, "{}", encoded_payload[0]).into_diagnostic()?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct PartialPayloads(Vec<PartialPayload>);

impl FromStr for PartialPayloads {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payloads = serde_json::from_str::<OneOrMany<PartialPayload>>(s)?;

        match payloads {
            OneOrMany::One(_) => Err(serde_json::Error::custom("Expected a list of payloads")),
            OneOrMany::Many(payloads) => Ok(Self(payloads)),
            OneOrMany::Empty => Err(serde_json::Error::custom("Input is empty")),
        }
    }
}
