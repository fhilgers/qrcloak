// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    fmt::Display,
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

#[cfg(test)]
use std::sync::{Arc, Mutex, Weak};

#[derive(Clone, Debug)]
pub enum FileOrStdout {
    File(PathBuf),
    Stdout,
    #[cfg(test)]
    InMem(Arc<Mutex<Vec<u8>>>),
}

#[cfg(test)]
impl FileOrStdout {
    pub fn new_testing() -> Self {
        FileOrStdout::InMem(Arc::new(Mutex::new(Vec::new())))
    }

    pub fn into_inner(self) -> Vec<u8> {
        match self {
            FileOrStdout::InMem(v) => {
                let v = Arc::into_inner(v).expect("should have only one reference");
                let v = v.into_inner().expect("should not be locked");
                v
            }
            _ => panic!("should be in-mem"),
        }
    }
}

impl Display for FileOrStdout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOrStdout::File(path) => write!(f, "{}", path.display()),
            FileOrStdout::Stdout => write!(f, "-"),
            #[cfg(test)]
            FileOrStdout::InMem(_) => write!(f, "<in-mem>"),
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
            #[cfg(test)]
            FileOrStdout::InMem(ref cell) => Ok(FileOrStdoutWriter::InMem(Arc::downgrade(cell))),
        }
    }
}

pub enum FileOrStdoutWriter {
    File(std::fs::File),
    Stdout(std::io::Stdout),
    #[cfg(test)]
    InMem(Weak<Mutex<Vec<u8>>>),
}

impl Write for FileOrStdoutWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            FileOrStdoutWriter::File(file) => file.write(buf),
            FileOrStdoutWriter::Stdout(stdout) => stdout.write(buf),
            #[cfg(test)]
            FileOrStdoutWriter::InMem(v) => {
                let v = v.upgrade().ok_or(io::Error::new(
                    io::ErrorKind::Other,
                    "unable to upgrade arc",
                ))?;
                let mut v = v.lock().map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("unable to lock arc: {}", e))
                })?;
                v.write(buf)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            FileOrStdoutWriter::File(file) => file.flush(),
            FileOrStdoutWriter::Stdout(stdout) => stdout.flush(),
            #[cfg(test)]
            FileOrStdoutWriter::InMem(_) => Ok(()),
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
