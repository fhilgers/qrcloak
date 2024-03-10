uniffi::setup_scaffolding!();

use std::{fmt::Debug, sync::Mutex};

use qrypt_core::*;
use quircs::{Code, DecodeError, ExtractError, Quirc};

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum MyError {
    #[error(transparent)]
    MyLibError(#[from] MyLibError),

    #[error(transparent)]
    ExtractError(#[from] ExtractError),

    #[error(transparent)]
    DecodeError(#[from] DecodeError),
}

#[uniffi::export]
pub fn is_payload(input: &str) -> bool {
    qrypt_core::is_payload(input)
}

#[uniffi::export]
pub fn encrypt_to_b64png(input: &str, password: &str) -> Result<String, MyError> {
    Ok(qrypt_core::encrypt_to_b64png(input, password)?)
}

#[uniffi::export]
pub fn encrypt_to_b64(input: &str, password: &str) -> Result<String, MyError> {
    Ok(qrypt_core::encrypt_to_b64(input, password)?)
}

#[uniffi::export]
pub fn decrypt_b64payload(input: &str, password: &str) -> Result<String, MyError> {
    Ok(qrypt_core::decrypt_b64payload(input, password)?)
}

#[derive(Debug, uniffi::Object)]
pub struct Decoder {
    inner: Mutex<Quirc>,
}

#[derive(Debug, uniffi::Enum)]
pub enum QrCode {
    Decoded { corners: Corners, payload: Vec<u8> },
    NotDecodable { corners: Corners },
}

impl From<&Code> for QrCode {
    fn from(value: &Code) -> Self {
        match value.decode() {
            Ok(v) => QrCode::Decoded {
                corners: value.corners.into(),
                payload: v.payload,
            },
            Err(_) => QrCode::NotDecodable {
                corners: value.corners.into(),
            },
        }
    }
}

impl From<Code> for QrCode {
    fn from(value: Code) -> Self {
        Self::from(&value)
    }
}

#[derive(Debug, Clone, Copy, uniffi::Record)]
pub struct Corners {
    pub top_left: Point,
    pub top_right: Point,
    pub bottom_right: Point,
    pub bottom_left: Point,
}

impl From<&[Point; 4]> for Corners {
    fn from(value: &[Point; 4]) -> Self {
        Self {
            top_left: value[0],
            top_right: value[1],
            bottom_right: value[2],
            bottom_left: value[3],
        }
    }
}

impl From<[Point; 4]> for Corners {
    fn from(value: [Point; 4]) -> Self {
        Self::from(&value)
    }
}

impl From<&[quircs::Point; 4]> for Corners {
    fn from(value: &[quircs::Point; 4]) -> Self {
        Self {
            top_left: value[0].into(),
            top_right: value[1].into(),
            bottom_right: value[2].into(),
            bottom_left: value[3].into(),
        }
    }
}

impl From<[quircs::Point; 4]> for Corners {
    fn from(value: [quircs::Point; 4]) -> Self {
        Self::from(&value)
    }
}

#[derive(Debug, Clone, Copy, uniffi::Record)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl From<&quircs::Point> for Point {
    fn from(value: &quircs::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<quircs::Point> for Point {
    fn from(value: quircs::Point) -> Self {
        Self::from(&value)
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl Decoder {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Quirc::default()),
        }
    }

    pub fn decode(&self, image: &[u8], width: u32, height: u32) -> Vec<QrCode> {
        let mut guard = self.inner.lock().unwrap();

        let codes = guard.identify(width as usize, height as usize, image);

        // We can unwrap as the only error that is thrown is when the count is out of
        // bounds. CodeIter<'_> checks that for us.
        codes
            .map(Result::unwrap)
            .map(QrCode::from)
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, uniffi::Record)]
pub struct EncodedQrCode {
    data: Vec<u8>,
    size: u16,
}

#[uniffi::export]
pub fn qr_encode(data: &str) -> Result<EncodedQrCode, MyError> {
    let qrcode = qrypt_core::qr_encode(data)?;

    let mut raw_data = Vec::with_capacity(qrcode.size() as usize * qrcode.size() as usize);

    for y in 0..qrcode.size() {
        for x in 0..qrcode.size() {
            if qrcode.get_module(x, y) {
                raw_data.push(1u8)
            } else {
                raw_data.push(0u8)
            }
        }
    }

    Ok(EncodedQrCode {
        data: raw_data,
        size: qrcode.size() as u16,
    })
}
