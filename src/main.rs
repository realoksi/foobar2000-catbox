mod catbox;
mod config;
mod encode;

use crate::catbox::{Catbox, Service};
use crate::config::{ClampTranscodeConfig, Config, TranscodeConfig};
use crate::encode::encode_image;

use anyhow::{Context, Result};
use image::{DynamicImage, guess_format, imageops::FilterType};
use std::{
    env, fs, io,
    io::{BufRead, stdin},
    path::{Path, PathBuf},
};

fn read_path_line(input: impl BufRead) -> Result<PathBuf> {
    let input = input.lines().next().with_context(|| "No input")??;
    Ok(input.trim().into())
}

fn load_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("Failed to read file: {:?}", path))
}

fn clamp(img: DynamicImage, clamp_config: &ClampTranscodeConfig) -> DynamicImage {
    let (max_width, max_height) = (
        clamp_config.max_resolution.width,
        clamp_config.max_resolution.height,
    );

    if img.width() > max_width || img.height() > max_height {
        img.resize(max_width, max_height, FilterType::CatmullRom)
    } else {
        img
    }
}

fn transcode(buf: &[u8], transcode_config: &TranscodeConfig) -> Result<Vec<u8>> {
    let format = guess_format(buf)?;
    let mut img = image::load_from_memory_with_format(buf, format)?;

    if let Some(clamp_config) = transcode_config.clamp {
        img = clamp(img, &clamp_config);
    }

    encode_image(&img, transcode_config)
}

fn get_config(path: PathBuf) -> Result<Config> {
    match fs::read_to_string(&path) {
        Ok(buf) => {
            toml::from_str(&buf).with_context(|| format!("Couldn't parse config file {:?}", path))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Default::default()),
        Err(e) => Err(e).with_context(|| format!("Couldn't read config file {:?}", path)),
    }
}

fn run(input: impl BufRead, config: Config, service: impl Service) -> Result<String> {
    let path = read_path_line(input)?;
    let file_buf = load_file(&path)?;

    let payload = match config.transcode {
        Some(transcode_config) => transcode(&file_buf, &transcode_config)?,
        None => file_buf,
    };

    service.upload_image(None, payload)
}

fn main() -> Result<()> {
    env_logger::init();

    let config_path = env::current_exe()?.parent().unwrap().join("config.toml");
    let config = get_config(config_path)?;
    let service = Catbox::new(
        None,
        config.user_hash.clone(),
        config.timeout,
        config.connection_timeout,
    )?;
    let response = run(stdin().lock(), config, service)?;

    println!("{}", response);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Format, JpgTranscodeConfig, MaxResolution};
    use image::{
        ColorType,
        codecs::png::{CompressionType, PngEncoder},
    };
    use std::{
        io::{BufReader, Cursor},
        str::FromStr,
    };

    const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

    fn temp_config_path() -> Result<PathBuf> {
        PathBuf::from_str(format!("{}/config.toml", CARGO_MANIFEST_DIR).as_str())
            .with_context(|| "Bad path")
    }

    /// creates a blank png
    fn temp_image_buf(w: u32, h: u32, color_type: ColorType) -> Vec<u8> {
        let img = DynamicImage::new(w, h, color_type);

        let mut cursor = Cursor::new(Vec::new());

        img.write_with_encoder(PngEncoder::new_with_quality(
            &mut cursor,
            CompressionType::Best,
            image::codecs::png::FilterType::Adaptive,
        ))
        .unwrap();

        cursor.into_inner()
    }

    #[test]
    fn test_read_path_line() {
        let input = BufReader::new("    /my/path/line/image.jpg    \n".as_bytes());

        let path_line = read_path_line(input).unwrap();

        assert_eq!(path_line.as_os_str(), "/my/path/line/image.jpg");
    }

    #[test]
    fn test_load_file() {
        load_file(&PathBuf::from(format!(
            "{}/config.toml",
            CARGO_MANIFEST_DIR
        )))
        .unwrap();
    }

    #[test]
    fn test_clamp() {
        let rgb8 = DynamicImage::new(32, 32, ColorType::Rgb8);
        let clamp_config = ClampTranscodeConfig {
            max_resolution: MaxResolution {
                width: 16,
                height: 16,
            },
        };

        let result_1 = clamp(rgb8, &clamp_config);

        assert_eq!(result_1.width(), 16);
        assert_eq!(result_1.height(), 16);

        let result_2 = clamp(result_1, &clamp_config);

        assert_eq!(result_2.width(), 16);
        assert_eq!(result_2.height(), 16);
    }

    #[test]
    fn test_transcode() {
        let buf = temp_image_buf(32, 32, ColorType::Rgb8);

        let transcode_config = TranscodeConfig {
            format: Format::Jpg,
            quality: 80,
            png: None,
            jpg: Some(JpgTranscodeConfig {
                ..Default::default()
            }),
            webp: None,
            clamp: Some(ClampTranscodeConfig {
                max_resolution: MaxResolution {
                    width: 16,
                    height: 16,
                },
            }),
        };

        transcode(&buf, &transcode_config).unwrap();
    }

    #[test]
    fn test_get_config() {
        let good_config_path = temp_config_path().unwrap();

        get_config(good_config_path).unwrap();
    }
}
