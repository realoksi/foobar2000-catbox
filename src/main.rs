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

fn get_config() -> Result<Config> {
    let config_path = env::current_exe()?.parent().unwrap().join("config.toml");

    match fs::read_to_string(&config_path) {
        Ok(buf) => toml::from_str(&buf)
            .with_context(|| format!("couldn't parse config file {:?}", config_path)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Default::default()),
        Err(e) => Err(e).with_context(|| format!("couldn't read config file {:?}", config_path)),
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let config = get_config()?;

    let file_buf = get_file_buf()?;
    let mut modified_buf = None;

    if let Some(transcode_config) = config.transcode {
        let format = guess_format(&file_buf)?;
        let mut image_buf = image::load_from_memory_with_format(&file_buf, format)?;

        if let Some(clamp_config) = transcode_config.clamp {
            let max_width = clamp_config.max_resolution.width;
            let max_height = clamp_config.max_resolution.height;

            if image_buf.width() > max_width || image_buf.height() > max_height {
                image_buf = image_buf.resize(max_width, max_height, FilterType::CatmullRom);
            }
        }

        modified_buf = Some(encode_image(&image_buf, &transcode_config)?);
    }

    let cb = Catbox::new(
        None,
        config.user_hash,
        config.timeout,
        config.connection_timeout,
    )?;

    let response = cb.upload_image(None, modified_buf.unwrap_or(file_buf))?;

    println!("{}", response);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config() {
        get_config().unwrap();
    }
}
