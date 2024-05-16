use std::{
    fmt::Debug,
    io::{self},
};

mod output;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};

pub use output::FileOrStdout;

use payload::PayloadCommand;

use qrcode::QrCodeCommand;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Payload(PayloadArgs),
    #[command(name = "qrcode")]
    QrCode(QrCodeArgs),

    #[command(hide = true)]
    Completion(CompletionArgs),
}

#[derive(Parser, Debug)]
struct CompletionArgs {
    shell: Shell,
}

#[derive(Parser, Debug)]
struct PayloadArgs {
    #[command(subcommand)]
    command: PayloadCommand,
}

#[derive(Parser, Debug)]
struct QrCodeArgs {
    #[command(subcommand)]
    inner: QrCodeCommand,
}

pub mod decryption;

pub mod encryption;
pub mod env;
pub mod input;
mod payload;
mod qrcode;

fn main() -> miette::Result<()> {
    miette::set_panic_hook();
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }))?;

    let cli = Cli::parse();

    match cli.command {
        Command::Completion(args) => {
            let cmd = &mut Cli::command();
            generate(
                args.shell,
                cmd,
                cmd.get_name().to_string(),
                &mut io::stdout(),
            );
        }
        Command::Payload(args) => match args.command {
            PayloadCommand::Generate(args) => args.handle()?,
            PayloadCommand::Extract(args) => args.handle()?,
            PayloadCommand::Merge(args) => args.handle()?,
        },
        Command::QrCode(args) => match args.inner {
            QrCodeCommand::Generate(args) => args.handle()?,
        },
    }

    Ok(())
}
