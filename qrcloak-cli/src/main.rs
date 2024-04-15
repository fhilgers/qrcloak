use std::{fmt::Debug, fs::create_dir_all, io, path::PathBuf, str::FromStr};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};

use encryption::EncryptionOptions;
use input::Input;
use miette::{miette, IntoDiagnostic};
use payload::PayloadCommand;
use qrcloak_core::{
    generate::Generator,
    payload::{format::Payload, OneOrMore, PayloadBuilder},
};
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
    #[arg(short, long, help = "Split payload into {} parts")]
    pub splits: Option<u32>,

    #[command(flatten)]
    encryption: EncryptionOptions,

    #[command(flatten)]
    input: Input<String>,

    #[arg(required = true, help = "Output files")]
    output: Vec<ImageFile>,
}

impl QrCodeGenerateArgs {
    pub fn handle(mut self) -> miette::Result<OneOrMore<'static, Payload>> {
        let input = self.input.contents().into_diagnostic()?;

        if self.splits.is_some() && self.output.len() > 1 {
            return Err(miette!("Cannod use splits with multiple output files"));
        } else {
            self.splits = Some(self.output.len() as u32);
        }

        let payloads = PayloadBuilder::default()
            .with_encryption(self.encryption.0)
            .with_splits(self.splits)
            .build(&input)
            .into_diagnostic()?;

        Ok(payloads)
    }
}

#[derive(Debug, Clone)]
struct ImageFile(PathBuf);

impl FromStr for ImageFile {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut path = PathBuf::from(s);

        if path.extension().is_none() {
            path.set_extension("png");
        }

        Ok(ImageFile(path))
    }
}

impl ImageFile {
    pub fn ensure_parent(&self) -> miette::Result<()> {
        if let Some(parent) = self.0.parent() {
            create_dir_all(parent).into_diagnostic()?;
        }
        Ok(())
    }

    pub fn into_path_iter(self, amount: Option<usize>) -> impl Iterator<Item = PathBuf> {
        let stem: PathBuf = self.0.file_stem().unwrap().into();
        let extension: PathBuf = self.0.extension().unwrap().into();

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
                let payloads = args.handle()?;

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
                let output = args.output.clone();
                let payloads = args.handle()?;

                let images = Generator::default().generate(&payloads).into_diagnostic()?;

                if images.is_one() {
                    assert_eq!(output.len(), 1);

                    let output = output.into_iter().next().unwrap();
                    output.ensure_parent()?;

                    let image = images.as_slice().into_iter().next().unwrap();
                    image.save(output.0).into_diagnostic()?;
                } else {
                    let amount = images.as_slice().len();

                    if output.len() == 1 {
                        let output = output.into_iter().next().unwrap();

                        output.ensure_parent()?;

                        for (image, path) in images
                            .as_slice()
                            .into_iter()
                            .zip(output.into_path_iter(Some(amount)))
                        {
                            image.save(path).into_diagnostic()?;
                        }
                    } else {
                        for (image, path) in images.as_slice().into_iter().zip(output.into_iter()) {
                            path.ensure_parent()?;
                            image.save(path.0).into_diagnostic()?;
                        }
                    };
                }
            }
        },
    }

    Ok(())
}
