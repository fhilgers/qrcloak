use std::str::FromStr;

use clap::Args;
use clap_stdin::{FileOrStdin, StdinError};

#[derive(Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub struct Input<T>
where
    T: FromStr + Clone + Send + Sync + 'static,
    <T as FromStr>::Err: std::error::Error + Sync + Send + 'static,
{
    #[arg(long, help = "Read data from argument")]
    pub text: Option<T>,

    #[arg(
        long,
        value_name = "FILENAME",
        help = "Read data from file (use '-' for stdin)"
    )]
    pub file: Option<FileOrStdin<T>>,
}

impl<T> Input<T>
where
    T: FromStr + Clone + Send + Sync + 'static,
    <T as FromStr>::Err: std::error::Error + Sync + Send + 'static,
{
    pub fn contents(self) -> Result<T, StdinError> {
        if let Some(text) = self.text {
            Ok(text)
        } else if let Some(file) = self.file {
            Ok(file.contents()?)
        } else {
            panic!("ArgGroup is required so at least one of the args should be set")
        }
    }
}
