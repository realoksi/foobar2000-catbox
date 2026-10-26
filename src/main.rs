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

    let config = get_config()?;
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

    #[test]
    fn test_get_config() {
        get_config().unwrap();
    }
}
