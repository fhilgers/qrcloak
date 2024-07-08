// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use quircs::Quirc;

use crate::format::Payload;

pub struct Extractor;

impl Extractor {
    pub fn extract(width: usize, height: usize, image: impl AsRef<[u8]>) -> Vec<Payload> {
        let mut decoder = Quirc::new();

        let codes = decoder.identify(width, height, image.as_ref());

        codes
            .into_iter()
            .filter_map(|code| code.ok())
            .filter_map(|code| code.decode().ok())
            .filter_map(|code| serde_json::from_slice::<Payload>(&code.payload).ok())
            .collect()
    }
}
