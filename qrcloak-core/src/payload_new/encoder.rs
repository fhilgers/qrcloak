use thiserror::Error;

use crate::{format::Payload, payload_new::one_or_more::OneOrMore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingOpts {
    Json { pretty: bool, merge: bool },
}

impl Default for EncodingOpts {
    fn default() -> Self {
        Self::Json {
            pretty: false,
            merge: false,
        }
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

    fn encode_json(
        &self,
        payloads: &OneOrMore<Payload>,
        pretty: bool,
    ) -> Result<String, EncodingError> {
        if pretty {
            Ok(serde_json::to_string_pretty(&payloads)?)
        } else {
            Ok(serde_json::to_string(&payloads)?)
        }
    }

    pub fn encode(
        &self,
        payloads: &OneOrMore<Payload>,
    ) -> Result<OneOrMore<'static, String>, EncodingError> {
        let mut result = Vec::with_capacity(payloads.len());

        match self.encoding_opts {
            EncodingOpts::Json { pretty, merge } => {
                if merge {
                    result.push(self.encode_json(payloads, pretty)?);
                } else {
                    for payload in payloads {
                        result.push(self.encode_json(&OneOrMore::from(payload), pretty)?);
                    }
                }
            }
        }

        Ok(OneOrMore::try_from(result).expect("at least one element"))
    }
}
