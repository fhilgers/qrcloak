// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::Subcommand;

pub use self::{
    extract::PayloadExtractArgs, generate::PayloadGenerateArgs, merge::PayloadMergeArgs,
};

mod extract;
mod generate;
mod merge;

#[derive(Subcommand, Debug)]
pub enum PayloadCommand {
    Generate(PayloadGenerateArgs),
    Extract(PayloadExtractArgs),
    Merge(PayloadMergeArgs),
}
