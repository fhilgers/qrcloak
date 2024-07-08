// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use thiserror::Error;

use crate::format::Payload;

use super::OneOrMany;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify, serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
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
        payloads: OneOrMany<Payload>,
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
        payloads: impl IntoIterator<Item = impl Into<Payload>>,
    ) -> Result<Vec<String>, EncodingError> {
        let payloads = payloads.into_iter().map(Into::into);

        let mut result = Vec::with_capacity(payloads.size_hint().0);

        match self.encoding_opts {
            EncodingOpts::Json { pretty, merge } => {
                if merge {
                    result.push(self.encode_json(payloads.collect(), pretty)?);
                } else {
                    for payload in payloads {
                        result.push(self.encode_json([payload].into_iter().collect(), pretty)?);
                    }
                }
            }
        }

        Ok(result)
    }
}
