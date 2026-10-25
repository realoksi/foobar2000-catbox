mod catbox;
mod config;
mod encode;

use crate::catbox::{Catbox, Service};
use crate::config::Config;
use crate::encode::encode_image;

use anyhow::{Context, Result, bail};
use image::{guess_format, imageops::FilterType};
use log::info;
use std::{
    env, fs, io,
    io::{BufRead, stdin},
    path::Path,
};

fn get_file_buf() -> Result<Vec<u8>> {
    info!("waiting for stdin");

    let input: String = stdin().lock().lines().next().unwrap()?.trim().to_string();

    let file_path = Path::new(&input);

    if !file_path.exists() {
        bail!("File not found");
    }

    info!("reading {:?}", file_path);

    Ok(fs::read(file_path)?)
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

#[test]
fn test_get_config() {
    get_config().unwrap();
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
                image_buf = image_buf.resize(max_width, max_height, FilterType::Nearest);
            }
        }

        modified_buf = Some(encode_image(&image_buf, &transcode_config)?);
    }

    let cb = Catbox::new(None, config.user_hash, config.timeout, config.connection_timeout)?;

    let response = cb.upload_image(None, modified_buf.unwrap_or(file_buf))?;

    println!("{}", response);

    Ok(())
}
