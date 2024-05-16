use clap::Subcommand;

mod generate;

pub use generate::QrCodeGenerateArgs;

#[derive(Subcommand, Debug)]
pub enum QrCodeCommand {
    Generate(QrCodeGenerateArgs),
}
