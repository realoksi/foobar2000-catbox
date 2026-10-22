use crate::config::{EncodeConfig, Format};
use anyhow::{Result, anyhow};
use image::{
    DynamicImage, EncodableLayout,
    codecs::{jpeg::JpegEncoder, png::PngEncoder},
};
use std::io::{Cursor, Write};

pub fn encode_image(img: &DynamicImage, encode_config: &EncodeConfig) -> Result<Vec<u8>> {
    let rgba8 = img.to_rgba8();
    let mut cursor = Cursor::new(Vec::new());

    match encode_config.format {
        Format::Png => {
            if let Some(png_config) = encode_config.png {
                rgba8.write_with_encoder(PngEncoder::new_with_quality(
                    &mut cursor,
                    png_config.compression.into(),
                    png_config.filter.into(),
                ))?;
            } else {
                rgba8.write_with_encoder(PngEncoder::new(&mut cursor))?;
            };
        }
        Format::Jpg => {
            rgba8.write_with_encoder(JpegEncoder::new_with_quality(
                &mut cursor,
                encode_config.quality().unwrap(),
            ))?;
        }
        Format::Webp => {
            let encoder = webp::Encoder::from_rgba(rgba8.as_bytes(), rgba8.width(), rgba8.height());

            let webp_memory = encoder
                .encode_simple(
                    encode_config.webp.unwrap_or_default().lossless,
                    encode_config.quality().unwrap() as f32,
                )
                .map_err(|e| anyhow!("{:?}", e))?;

            cursor.write_all(&webp_memory)?;
        }
    }

    Ok(cursor.into_inner())
}
