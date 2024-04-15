use thiserror::Error;

use super::{format::Payload, one_or_more::OneOrMore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingOpts {
    Json { pretty: bool },
}

impl Default for EncodingOpts {
    fn default() -> Self {
        Self::Json { pretty: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Encoder {
    encoding_opts: EncodingOpts,
}

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("transparent")]
    SerdeJsonError(#[from] serde_json::Error),
}

impl Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_encoding(mut self, encoding_opts: EncodingOpts) -> Self {
        self.encoding_opts = encoding_opts;
        self
    }

    pub fn encode(
        &self,
        payloads: &OneOrMore<Payload>,
    ) -> Result<OneOrMore<'static, String>, EncodingError> {
        let mut result = Vec::with_capacity(payloads.as_slice().len());

        match self.encoding_opts {
            EncodingOpts::Json { pretty } => {
                if pretty {
                    for payload in payloads.as_slice() {
                        result.push(serde_json::to_string_pretty(&payload)?);
                    }
                } else {
                    for payload in payloads.as_slice() {
                        result.push(serde_json::to_string(&payload)?);
                    }
                }
            }
        }

        Ok(OneOrMore::try_from(result).expect("at least one element"))
    }
}
