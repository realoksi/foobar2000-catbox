use std::{
    env, fs,
    io::{self, BufRead, Cursor},
    path::Path,
    process::exit,
};

use consumer::{Catbox, Consumer, Litterbox};
use image::codecs::{jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder};
use settings::{EncodeFormat, Error, Settings};

mod consumer;
mod settings;

fn main() {
    let settings_path: &Path = &env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("settings.yml");

    let settings = Settings::from_str(&fs::read_to_string(settings_path).unwrap_or_default())
        .unwrap_or(Settings::new());

    if let Err(e) = settings.validate() {
        match e {
            Error::EncodeFormatQualityError(_) => {
                eprintln!("\x1b[38;2;255;18;73m{}\x1b[0m", e);
                exit(1);
            }
            Error::ResizeMaxResolutionZeroError(_) => {
                eprintln!("\x1b[38;2;255;18;73m{}\x1b[0m", e);
                exit(2);
            }
            Error::ResizeMaxResolutionPowerError(_) => {
                eprintln!("\x1b[38;2;255;18;73m{}\x1b[0m", e);
                exit(3);
            }
            Error::UserHashError(_) => {
                eprintln!("\x1b[38;2;255;18;73m{}\x1b[0m", e);
                exit(4);
            }
        }
    }

    let input: String = io::stdin()
        .lock()
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .trim()
        .to_string();

    let file_path: &Path = Path::new(&input);

    if !file_path.exists() {
        eprintln!("{:?} doesn't exist.", file_path.as_os_str());
        exit(4);
    }

    let file_buffer: Vec<u8> = std::fs::read(&file_path).unwrap();

    let image_buffer = image::load_from_memory(&file_buffer).unwrap_or_else(|_| {
        eprintln!("Failed to load image from memory");
        exit(5);
    });

    let max_width = settings.resize_max_resolution[0];
    let max_height = settings.resize_max_resolution[1];

    let next_buffer = if settings.enable_resize
        && (image_buffer.width() > max_width || image_buffer.height() > max_height)
    {
        image_buffer.resize(max_width, max_height, image::imageops::FilterType::Nearest)
    } else {
        image_buffer
    };

    let mut cursor = Cursor::new(Vec::<u8>::new());

    if settings.enable_encode {
        match settings.encode_format {
            EncodeFormat::JPG => next_buffer
                .write_with_encoder(JpegEncoder::new_with_quality(
                    &mut cursor,
                    settings.encode_format_quality,
                ))
                .unwrap(),
            EncodeFormat::PNG => next_buffer
                .write_with_encoder(PngEncoder::new(&mut cursor))
                .unwrap(),
            EncodeFormat::WEBP => next_buffer
                .write_with_encoder(WebPEncoder::new_lossless(&mut cursor))
                .unwrap(),
        }
    } else {
        next_buffer
            .write_to(&mut cursor, image::guess_format(&file_buffer).unwrap())
            .unwrap();
    }

    let consumer: Box<dyn Consumer> = match settings.enable_litterbox {
        true => Box::new(Litterbox::new(
            settings.litterbox_expire_time.to_string(),
            Some(settings.user_agent),
            Some(settings.user_hash),
        )),
        false => Box::new(Catbox::new(
            Some(settings.user_agent),
            Some(settings.user_hash),
        )),
    };

    match consumer.upload_image(cursor.into_inner()) {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(6)
        }
    };
}
