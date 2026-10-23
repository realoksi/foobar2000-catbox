fn main() {
mod catbox;
mod config;
mod encode;

use crate::catbox::{Catbox, Service};
use crate::config::Config;
use crate::encode::encode_image;

use anyhow::{Result, bail};
use std::{
    io::{BufRead, stdin},
    path::Path,
};

fn get_file_buf() -> Result<Vec<u8>> {
    info!("waiting for stdin");

    let input: String = stdin().lock().lines().next().unwrap()?.trim().to_string();

    let file_path: &Path = Path::new(&input);

    if !file_path.exists() {
        bail!("File not found");
    }

    Ok(std::fs::read(&file_path)?)
    info!("reading {:?}", file_path);

}

fn main() -> Result<()> {

    Ok(())
}
