pub mod payload;

#[cfg(feature = "generate")]
pub mod generate;

#[cfg(feature = "extract")]
pub mod extract;

#[cfg(all(test, feature = "extract", feature = "generate"))]
mod tests {
    use image::GenericImage;

    use crate::{
        extract::Extractor,
        generate::Generator,
        payload::{extract_chains, PayloadBuilder},
    };

    #[test]
    fn test_simple() {
        let payload = PayloadBuilder::default()
            .with_splits(Some(4))
            .build("hello world")
            .expect("should build");

        let images = Generator::default()
            .generate(&payload)
            .expect("should generate")
            .as_slice()
            .into_iter()
            .map(|image| {
                image::imageops::resize(
                    image,
                    image.width() * 4,
                    image.height() * 4,
                    image::imageops::FilterType::Nearest,
                )
            })
            .collect::<Vec<_>>();

        let spacing = 16;
        let mut total_width = 0;
        let mut total_height = 0;
        for image in images.iter() {
            total_width += image.width() + spacing;
            total_height = total_height.max(image.height());
        }

        let mut pos = 0;
        let mut total_image = image::GrayImage::new(total_width, total_height);
        for image in images.into_iter() {
            total_image.copy_from(&image, pos, 0).expect("should copy");
            pos += image.width() + spacing;
        }

        let payloads = Extractor::extract(
            total_image.width() as usize,
            total_image.height() as usize,
            &*total_image,
        );

        let (complete, partial) = extract_chains(payloads);

        assert_eq!(complete.len(), 1);
        assert_eq!(partial.len(), 0);

        assert_eq!(&*complete[0].data, b"hello world");
    }
}
