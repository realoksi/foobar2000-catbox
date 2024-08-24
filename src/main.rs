use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, BufRead, Cursor},
    path::Path,
    process::exit,
};

use audiotags::{AudioTag, Tag};
use curl::easy::{Easy2, Form, Handler, List, WriteError};
use image::codecs::jpeg::JpegEncoder;

struct ResponseBody(Vec<u8>);

impl Handler for ResponseBody {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

mod error {
    pub const FILE_NOT_FOUND: i32 = 1;
    // pub const FILE_SYSTEM_READ_ERROR: i32 = 2;
    pub const IMAGE_LOADING_ERROR: i32 = 3;
    pub const IMAGE_ENCODING_ERROR: i32 = 4;
    pub const HTTP_REQUEST_ERROR: i32 = 5;
    pub const HTTP_RESPONSE_ERROR: i32 = 6;
}

fn main() {
    // Configuration map initialization

    let config_path: &Path = &env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.txt");
    let mut config: HashMap<String, String> = HashMap::new();

    if let Ok(config_file) = File::open(&config_path) {
        let config_reader: io::BufReader<File> = io::BufReader::new(config_file);

        for line in config_reader.lines() {
            if let Ok(line) = line {
                if let Some((key, value)) = line.split_once("=") {
                    config.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    // Definitions for default values

    const DEFAULT_MAX_WIDTH: u32 = 500;
    const DEFAULT_MAX_HEIGHT: u32 = 500;
    const DEFAULT_QUALITY: u8 = 80;
    let default_user_agent: String =
        "Mozilla/5.0 (X11; Linux x86_64; rv:123.0) Gecko/20100101 Firefox/123.0".to_string();
    let default_endpoint: String = "https://catbox.moe/user/api.php".to_string();

    // Configuration value initialization
    // When a value isn't available or is invalid, these will always default to their appropriate hardcoded value above.

    let max_width: u32 = config
        .get("MAX_WIDTH")
        .unwrap_or(&DEFAULT_MAX_WIDTH.to_string())
        .parse()
        .unwrap_or(DEFAULT_MAX_WIDTH);
    let max_height: u32 = config
        .get("MAX_HEIGHT")
        .unwrap_or(&DEFAULT_MAX_HEIGHT.to_string())
        .parse()
        .unwrap_or(DEFAULT_MAX_HEIGHT);
    let quality: u8 = config
        .get("QUALITY")
        .unwrap_or(&DEFAULT_QUALITY.to_string())
        .parse()
        .unwrap_or(DEFAULT_QUALITY);
    let user_agent: &String = config.get("USER_AGENT").unwrap_or(&default_user_agent);
    let endpoint: &String = config.get("ENDPOINT").unwrap_or(&default_endpoint);

    // ...

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
        exit(error::FILE_NOT_FOUND);
    }

    let file_name: &str = file_path.file_name().unwrap().to_str().unwrap();

    let file_buffer: Vec<u8> = Tag::new()
        .read_from_path(file_path)
        .and_then(|file_tag: Box<dyn AudioTag + Send + Sync>| {
            Ok(file_tag.album_cover().unwrap().data.to_vec())
        })
        .unwrap_or_else(|_| std::fs::read(&file_path).unwrap());

    let image_buffer: image::DynamicImage =
        image::load_from_memory(&file_buffer).unwrap_or_else(|_| {
            eprintln!("Failed to load image from memory");
            exit(error::IMAGE_LOADING_ERROR);
        });

    let resize_buffer: image::DynamicImage =
        if image_buffer.width() > max_width || image_buffer.height() > max_height {
            image_buffer.resize(max_width, max_height, image::imageops::FilterType::Nearest)
        } else {
            image_buffer
        };

    let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

    JpegEncoder::new_with_quality(&mut cursor, quality)
        .encode_image(&resize_buffer)
        .unwrap_or_else(|_| {
            eprintln!("Failed to encode image");
            exit(error::IMAGE_ENCODING_ERROR);
        });

    let mut form: Form = Form::new();
    form.part("reqtype").contents(b"fileupload").add().unwrap();
    form.part("userhash").contents(b"").add().unwrap();
    form.part("fileToUpload")
        .content_type("image/jpeg")
        .buffer(&file_name, cursor.into_inner())
        .add()
        .unwrap();

    let mut headers: List = List::new();
    headers.append("Content-Type: multipart/form-data").unwrap();
    headers
        .append(format!("User-Agent: {}", user_agent).as_str())
        .unwrap();

    let mut easy: Easy2<ResponseBody> = Easy2::new(ResponseBody(Vec::new()));
    easy.url(endpoint).unwrap();
    easy.http_headers(headers).unwrap();
    easy.httppost(form).unwrap();

    match easy.perform() {
        Ok(_) => {
            let response_code: u32 = easy.response_code().unwrap();

            if response_code == 200 || response_code == 304 {
                println!("{}", String::from_utf8_lossy(easy.get_ref().0.as_slice()));
            } else {
                eprintln!("Response error {}", response_code);
                exit(error::HTTP_RESPONSE_ERROR);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(error::HTTP_REQUEST_ERROR);
        }
    }
}
