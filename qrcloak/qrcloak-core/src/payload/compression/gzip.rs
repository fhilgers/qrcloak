// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::io;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use flate2::Compression;

#[derive(Debug, Clone)]
pub struct GzipCompression;

impl GzipCompression {
    pub fn compress(&self, data: Bytes) -> Bytes {
        let mut compress = flate2::bufread::GzEncoder::new(data.reader(), Compression::default());

        let mut output = BytesMut::new().writer();
        io::copy(&mut compress, &mut output).expect("copying failed");

        output.into_inner().freeze()
    }

    pub fn decompress(&self, data: Bytes) -> Bytes {
        let mut decompress = flate2::bufread::GzDecoder::new(data.reader());

        let mut output = BytesMut::new().writer();
        io::copy(&mut decompress, &mut output).expect("copying failed");

        output.into_inner().freeze()
    }
}
