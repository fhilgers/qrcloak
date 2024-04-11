use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};

use miette::{Context, IntoDiagnostic};
use qrcloak_core::{
    generate::Generator,
    payload::{format::Payload, PayloadBuilder},
};
use std::io::Write;

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
enum PayloadCommand {
    Generate(PayloadGenerateArgs),
}

#[derive(Subcommand, Debug)]
enum QrCodeCommand {
    Generate(QrCodeGenerateArgs),
}

#[derive(Parser, Debug)]
struct QrCodeGenerateArgs {
    #[command(flatten)]
    payload_args: PayloadGenerateArgs,

    output: PathBuf,
}

#[derive(Parser, Debug)]
struct PayloadGenerateArgs {
    #[arg(short, long, default_value_t = false)]
    from_file: bool,

    #[arg(short, long)]
    splits: Option<u32>,

    input: String,
}

fn handle_payload_generate(args: PayloadGenerateArgs) -> miette::Result<Vec<Payload>> {
    let input = if args.from_file {
        let path = PathBuf::from(args.input);
        let mut file = File::open(path)
            .into_diagnostic()
            .wrap_err("Could not find payload file")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .into_diagnostic()
            .wrap_err("Could not read payload file")?;
        buf
    } else {
        args.input.as_bytes().to_vec()
    };

    if let Some(splits) = args.splits {
        PayloadBuilder::default()
            .build_partial(&input, splits)
            .into_diagnostic()
    } else {
        PayloadBuilder::default()
            .build_complete(&input)
            .into_diagnostic()
            .map(|p| vec![p])
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
                let payloads = handle_payload_generate(args)?;
                serde_json::to_writer_pretty(std::io::stdout(), &payloads).into_diagnostic()?;
                writeln!(std::io::stdout()).into_diagnostic()?;
            }
        },
        Command::QrCode(args) => match args.inner {
            QrCodeCommand::Generate(args) => {
                let mut output = args.output;

                if output.extension().is_none() {
                    output.set_extension("png");
                }

                let payloads = handle_payload_generate(args.payload_args)?;

                let images = Generator::default()
                    .generate_many(&payloads)
                    .into_diagnostic()?;

                if images.len() == 1 {
                    let image = images.into_iter().next().unwrap();
                    image.save(output).into_diagnostic()?;
                } else {
                    let stem: PathBuf = output.file_stem().unwrap().into();
                    let extension: PathBuf = output.extension().unwrap().into();

                    for (i, image) in images.into_iter().enumerate() {
                        image
                            .save(PathBuf::from(format!(
                                "{}-{}.{}",
                                stem.display(),
                                i,
                                extension.display()
                            )))
                            .into_diagnostic()?;
                    }
                }
            }
        },
    }

    Ok(())
}
