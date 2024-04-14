use std::str::FromStr;

use clap::Parser;

use miette::{Context, IntoDiagnostic};
use qrcloak_core::payload::format::{CompletePayload, PartialPayload};
use serde::de::Error;

use crate::{input::Input, OneOrMany};

#[derive(Parser, Debug)]
pub struct PayloadMergeArgs {
    #[command(flatten)]
    input: Input<PartialPayloads>,
}

impl PayloadMergeArgs {
    pub fn handle(self) -> miette::Result<CompletePayload> {
        let payloads = self
            .input
            .contents()
            .into_diagnostic()
            .wrap_err("Unable to read payloads from input")?;

        let complete_payload =
            qrcloak_core::payload::convert_chain(payloads.0).into_diagnostic()?;

        Ok(complete_payload)
    }
}

#[derive(Clone, Debug)]
struct PartialPayloads(Vec<PartialPayload>);

impl FromStr for PartialPayloads {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payloads = serde_json::from_str::<OneOrMany<PartialPayload>>(s)?;

        match payloads {
            OneOrMany::One(payload) => Ok(PartialPayloads(vec![payload])),
            OneOrMany::Many(payloads) => {
                if payloads.len() == 0 {
                    Err(serde_json::Error::custom("No payloads found"))
                } else {
                    Ok(PartialPayloads(payloads))
                }
            }
        }
    }
}
