use std::{fmt::Debug, io, path::PathBuf, str::FromStr};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};

use miette::IntoDiagnostic;
use payload::{PayloadCommand, PayloadGenerateArgs};
use qrcloak_core::{generate::Generator, payload::format::Payload};
use serde::{Deserialize, Serialize};

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

#[derive(Subcommand, Debug)]
enum QrCodeCommand {
    Generate(QrCodeGenerateArgs),
}

#[derive(Parser, Debug)]
struct QrCodeGenerateArgs {
    #[command(flatten)]
    payload_args: PayloadGenerateArgs,

    output: ImageFile,
}

#[derive(Debug, Clone)]
struct ImageFile(PathBuf);

impl FromStr for ImageFile {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ImageFile(PathBuf::from(s)))
    }
}

impl ImageFile {
    pub fn into_path(mut self) -> PathBuf {
        if self.0.extension().is_none() {
            self.0.set_extension("png");
        }

        self.0
    }

    pub fn into_path_iter(self, amount: Option<usize>) -> impl Iterator<Item = PathBuf> {
        let path = self.into_path();

        let stem: PathBuf = path.file_stem().unwrap().into();
        let extension: PathBuf = path.extension().unwrap().into();

        let max_width = if let Some(amt) = amount {
            format!("{amt}").len()
        } else {
            0
        };

        (0..).map(move |i| {
            PathBuf::from(format!(
                "{}-{:0>max_width$}.{}",
                stem.display(),
                i,
                extension.display()
            ))
        })
    }
}

pub mod decryption;

pub mod encryption;
pub mod env;
pub mod input;
mod payload;
mod qrcode;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(v: Vec<T>) -> Self {
        if v.len() == 1 {
            OneOrMany::One(v.into_iter().next().unwrap())
        } else {
            OneOrMany::Many(v)
        }
    }
}

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
            PayloadCommand::Generate(args) => {
                let payloads: OneOrMany<Payload> = args.handle()?.into();

                serde_json::to_writer_pretty(std::io::stdout(), &payloads).into_diagnostic()?;

                println!()
            }
            PayloadCommand::Extract(args) => {
                let text = args.handle()?;

                println!("{}", text);
            }
            PayloadCommand::Merge(args) => {
                let complete_payload = args.handle()?;
                serde_json::to_writer_pretty(std::io::stdout(), &complete_payload)
                    .into_diagnostic()?;

                println!()
            }
        },
        Command::QrCode(args) => match args.inner {
            QrCodeCommand::Generate(args) => {
                let payloads = args.payload_args.handle()?;

                let images = Generator::default()
                    .generate_many(&payloads)
                    .into_diagnostic()?;

                if images.len() == 1 {
                    let image = images.into_iter().next().unwrap();
                    image.save(args.output.into_path()).into_diagnostic()?;
                } else {
                    let amount = images.len();

                    for (image, path) in images
                        .into_iter()
                        .zip(args.output.into_path_iter(Some(amount)))
                    {
                        image.save(path).into_diagnostic()?;
                    }
                }
            }
        },
    }

    Ok(())
}
