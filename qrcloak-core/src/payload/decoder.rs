use thiserror::Error;

use crate::{format::Payload, payload::one_or_more::OneOrMore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodingOpts {
    Json,
}

impl Default for DecodingOpts {
    fn default() -> Self {
        Self::Json
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Decoder {
    decoding_opts: DecodingOpts,
}

#[derive(Debug, Error)]
pub enum DecodingError {
    #[error("transparent")]
    SerdeJsonError(#[from] serde_json::Error),
}

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_opts(mut self, decoding_opts: DecodingOpts) -> Self {
        self.decoding_opts = decoding_opts;
        self
    }

    fn decode_json(&self, data: &[u8]) -> Result<OneOrMore<'static, Payload>, DecodingError> {
        Ok(serde_json::from_slice(data)?)
    }

    pub fn decode(&self, data: &[u8]) -> Result<OneOrMore<'static, Payload>, DecodingError> {
        let payloads = match self.decoding_opts {
            DecodingOpts::Json => self.decode_json(data)?,
        };

        Ok(payloads)
    }
}
