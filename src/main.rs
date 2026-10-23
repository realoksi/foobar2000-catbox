fn main() {
mod catbox;
mod config;
mod encode;

use crate::catbox::{Catbox, Service};
use crate::config::Config;
use crate::encode::encode_image;

use anyhow::{Result, bail};
use image::guess_format;
use log::info;
use std::{
    env, fs,
    io::{BufRead, Cursor, stdin},
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

fn main() -> Result<()> {
    env_logger::init();

    let config_path = env::current_exe()?.parent().unwrap().join("config.toml");

    let config = if config_path.exists() {
        info!("using config from disk");
        toml::from_str::<Config>(fs::read_to_string(&config_path)?.as_str())?.validate()?
    } else {
        info!("fallback to default config");
        Config::default()
    };

    let file_buf = get_file_buf()?;
    let format = guess_format(file_buf.as_slice())?;
    let image_buf = image::load_from_memory_with_format(file_buf.as_slice(), format)?;

    // TODO: Add a resize block *before* encoding.

    let last_buf = if config.encode.enabled {
        encode_image(&image_buf, &config.encode)?
    } else {
        let mut cursor = Cursor::new(Vec::new());

        image_buf.write_to(&mut cursor, format)?;
        cursor.into_inner()
    };

    let cb = Catbox::new(None, config.catbox.user_hash)?;

    let response = cb.upload_image(last_buf)?;

    println!("{}", response);

    Ok(())
}
