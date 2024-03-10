use std::io::{Read, Write};

use age::{secrecy::SecretString, DecryptError, Decryptor, EncryptError, Encryptor};
use base64::{DecodeError, Engine};
use image::{codecs::png::PngEncoder, ImageError, Rgba, RgbaImage};

#[cfg(feature = "miette")]
use miette::Diagnostic;

use qrcodegen::QrCode;
use thiserror::Error;

pub fn encrypt(input: &str, password: &str) -> Result<Vec<u8>, EncryptError> {
    let encryptor = Encryptor::with_user_passphrase(SecretString::new(password.into()));

    let mut output = vec![];

    let mut writer = encryptor.wrap_output(&mut output)?;

    writer.write_all(input.as_bytes())?;
    writer.finish()?;

    Ok(output)
}

pub fn decrypt(input: &[u8], password: &str) -> Result<String, DecryptError> {
    let decryptor = match Decryptor::new(input)? {
        Decryptor::Passphrase(pp) => pp,
        Decryptor::Recipients(_) => {
            return Err(DecryptError::UnknownFormat);
        }
    };

    let mut reader = decryptor.decrypt(&SecretString::new(password.to_string()), None)?;

    let mut result = String::new();
    reader.read_to_string(&mut result)?;

    Ok(result)
}

pub fn b64_encode(input: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(input)
}

pub fn b64_decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    base64::engine::general_purpose::STANDARD.decode(input)
}

pub fn qr_encode(input: &str) -> Result<QrCode, MyLibError> {
    Ok(QrCode::encode_text(input, qrcodegen::QrCodeEcc::Low)?)
}

pub fn to_png(qr: &QrCode) -> Result<Vec<u8>, ImageError> {
    let mut img = RgbaImage::new((qr.size() * 4) as u32, (qr.size() * 4) as u32);

    for y in 0..img.height() {
        for x in 0..img.width() {
            if qr.get_module((x / 4) as i32, (y / 4) as i32) {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]))
            } else {
                img.put_pixel(x, y, Rgba([0, 0, 0, 0]))
            }
        }
    }

    let mut buf = vec![];

    let png_encoder = PngEncoder::new(&mut buf);
    img.write_with_encoder(png_encoder)?;

    Ok(buf)
}

#[cfg_attr(feature = "miette", derive(Diagnostic))]
#[derive(Error, Debug)]
pub enum MyLibError {
    #[error(transparent)]
    EncryptError(#[from] age::EncryptError),

    #[error(transparent)]
    DecryptError(#[from] age::DecryptError),

    #[error(transparent)]
    DataTooLong(#[from] qrcodegen::DataTooLong),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    DecodeError(#[from] base64::DecodeError),
}

const IDENTIFIER: &[u8; 16] = b"PWEncrypt001\0\0\0\0";

pub fn is_payload(input: &str) -> bool {
    if let Ok(decoded) = b64_decode(input) {
        has_identifier(&decoded)
    } else {
        false
    }
}

pub fn has_identifier(input: &[u8]) -> bool {
    input.starts_with(IDENTIFIER)
}

pub fn strip_identifier(input: &[u8]) -> Option<&[u8]> {
    input.strip_prefix(IDENTIFIER)
}

pub fn encrypt_to_b64(input: &str, password: &str) -> Result<String, MyLibError> {
    let encrypted = encrypt(input, password)?;
    let encoded = b64_encode(&[&IDENTIFIER[..], &encrypted[..]].concat());

    Ok(encoded)
}

pub fn encrypt_to_b64png(input: &str, password: &str) -> Result<String, MyLibError> {
    let encoded = encrypt_to_b64(input, password)?;
    let qrcode = qr_encode(&encoded)?;
    let png = to_png(&qrcode)?;
    let b64_png = b64_encode(&png);

    Ok(b64_png)
}

pub fn decrypt_b64payload(input: &str, password: &str) -> Result<String, MyLibError> {
    let mut decoded = &b64_decode(input)?[..];

    if let Some(remaining) = strip_identifier(decoded) {
        decoded = remaining
    }

    let decrypted = decrypt(decoded, password)?;

    Ok(decrypted)
}
