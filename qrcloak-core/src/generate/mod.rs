use image::{GrayImage, ImageBuffer, Luma};
use qrcodegen::{QrCode, Version};
use thiserror::Error;

use crate::format::Payload;

#[derive(Debug, Clone, Copy, Default)]
pub enum Encoding {
    #[default]
    Json,
}

#[derive(Debug, Clone)]
pub struct Generator {
    encoding: Encoding,
    ecl: qrcodegen::QrCodeEcc,
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            encoding: Encoding::default(),
            ecl: qrcodegen::QrCodeEcc::High,
        }
    }
}

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("transparent")]
    DataTooLong(#[from] qrcodegen::DataTooLong),

    #[error("transparent")]
    SerializationError(#[from] serde_json::Error),
}

fn qrcode_to_image(qrcode: &QrCode) -> GrayImage {
    let scale = 4;

    let size = qrcode.size() as u32;

    let mut img = ImageBuffer::new(size * scale + scale * 2, size * scale + scale * 2);

    img.fill(255);

    for y in 0..size {
        for x in 0..size {
            if qrcode.get_module(x as i32, y as i32) {
                let scaled_y_with_offset = y * scale + scale;
                let scaled_x_with_offset = x * scale + scale;

                for real_y in scaled_y_with_offset..scaled_y_with_offset + scale {
                    for real_x in scaled_x_with_offset..scaled_x_with_offset + scale {
                        img.put_pixel(real_x, real_y, Luma([0]))
                    }
                }
            }
        }
    }

    img.into()
}

impl Generator {
    pub fn with_encoding(self, encoding: Encoding) -> Self {
        Self { encoding, ..self }
    }

    pub fn with_ecl(self, ecl: qrcodegen::QrCodeEcc) -> Self {
        Self { ecl, ..self }
    }

    pub fn generate(
        &self,
        payload: impl IntoIterator<Item = impl Into<Payload>>,
    ) -> Result<Vec<GrayImage>, GenerateError> {
        let iter = payload.into_iter().map(Into::into);

        self.generate_many(iter)
    }

    fn generate_many(
        &self,
        payloads: impl Iterator<Item = Payload>,
    ) -> Result<Vec<GrayImage>, GenerateError> {
        match self.encoding {
            Encoding::Json => {
                let mut minversion = Version::MIN;

                let mut result = Vec::with_capacity(payloads.size_hint().0);

                for payload in payloads {
                    let json = serde_json::to_string(&payload)?;
                    let qrcode = qrcodegen::QrCode::encode_text_optimally_advanced(
                        &json,
                        self.ecl,
                        minversion,
                        Version::MAX,
                        None,
                        false,
                    )?;
                    let image = qrcode_to_image(&qrcode);
                    result.push(image);

                    minversion = qrcode.version()
                }

                Ok(result)
            }
        }
    }
}
