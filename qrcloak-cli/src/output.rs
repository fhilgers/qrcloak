use std::{
    fmt::Display,
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Clone, Debug)]
pub enum FileOrStdout {
    File(PathBuf),
    Stdout,
}

impl Display for FileOrStdout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOrStdout::File(path) => write!(f, "{}", path.display()),
            FileOrStdout::Stdout => write!(f, "-"),
        }
    }
}

impl FileOrStdout {
    pub fn try_get_writer(&self) -> Result<FileOrStdoutWriter, io::Error> {
        FileOrStdoutWriter::try_from(self.clone())
    }
}

impl TryFrom<FileOrStdout> for FileOrStdoutWriter {
    type Error = io::Error;

    fn try_from(value: FileOrStdout) -> Result<Self, Self::Error> {
        match value {
            FileOrStdout::File(path) => Ok(FileOrStdoutWriter::File(std::fs::File::create(path)?)),
            FileOrStdout::Stdout => Ok(FileOrStdoutWriter::Stdout(std::io::stdout())),
        }
    }
}

pub enum FileOrStdoutWriter {
    File(std::fs::File),
    Stdout(std::io::Stdout),
}

impl Write for FileOrStdoutWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            FileOrStdoutWriter::File(file) => file.write(buf),
            FileOrStdoutWriter::Stdout(stdout) => stdout.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            FileOrStdoutWriter::File(file) => file.flush(),
            FileOrStdoutWriter::Stdout(stdout) => stdout.flush(),
        }
    }
}

impl FromStr for FileOrStdout {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(FileOrStdout::Stdout)
        } else {
            Ok(FileOrStdout::File(PathBuf::from(s)))
        }
    }
}
