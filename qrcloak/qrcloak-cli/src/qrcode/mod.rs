// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::Subcommand;

mod generate;

pub use generate::QrCodeGenerateArgs;

#[derive(Subcommand, Debug)]
pub enum QrCodeCommand {
    Generate(QrCodeGenerateArgs),
}
