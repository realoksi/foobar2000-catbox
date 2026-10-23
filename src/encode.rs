use crate::config::{EncodeConfig, Format, PngEncodeConfig};
use anyhow::{Result, anyhow};
use image::{
    DynamicImage, EncodableLayout,
    codecs::{jpeg::JpegEncoder, png::PngEncoder},
};
use log::info;
use std::io::Cursor;

fn encode_png(buf: &DynamicImage, png_config: &PngEncodeConfig) -> Result<Vec<u8>> {
    info!("encoding as png");

    let mut cursor = Cursor::new(Vec::new());
    let rgba8 = buf.to_rgba8();

    rgba8.write_with_encoder(PngEncoder::new_with_quality(
        &mut cursor,
        png_config.compression.into(),
        png_config.filter.into(),
    ))?;

    Ok(cursor.into_inner())
}

fn encode_jpg(buf: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
    info!("encoding as jpg");

    let mut cursor = Cursor::new(Vec::new());
    let rgb8 = buf.to_rgb8();

    rgb8.write_with_encoder(JpegEncoder::new_with_quality(&mut cursor, quality))?;

    Ok(cursor.into_inner())
}

fn encode_webp(buf: &DynamicImage, quality: f32, lossless: bool) -> Result<Vec<u8>> {
    info!("encoding as webp");

    let rgba8 = buf.to_rgba8();

    let encoder = webp::Encoder::from_rgba(rgba8.as_bytes(), rgba8.width(), rgba8.height());

    let webp_memory = encoder
        .encode_simple(lossless, quality)
        .map_err(|e| anyhow!("WebP encode failed: {:?}", e))?;

    Ok(webp_memory.to_vec())
}

pub fn encode_image(buf: &DynamicImage, encode_config: &EncodeConfig) -> Result<Vec<u8>> {
    match encode_config.format {
        Format::Png => encode_png(buf, &encode_config.png.unwrap_or_default()),
        Format::Jpg => encode_jpg(buf, encode_config.quality().unwrap()),
        Format::Webp => encode_webp(
            buf,
            encode_config.quality().unwrap().into(),
            encode_config.webp.unwrap_or_default().lossless,
        ),
    }
}
