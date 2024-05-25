use std::{
    io::{self, BufRead, Cursor},
    path::Path,
    process::exit,
};

use curl::easy::{Easy2, Form, Handler, List, WriteError};
use image::codecs::jpeg::JpegEncoder;

struct ResponseBody(Vec<u8>);

impl Handler for ResponseBody {
    // I stole this. Sorry
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

mod error {
    pub const FILE_NOT_FOUND: i32 = 1;
    pub const FILE_SYSTEM_READ_ERROR: i32 = 2;
    pub const IMAGE_LOADING_ERROR: i32 = 3;
    pub const IMAGE_ENCODING_ERROR: i32 = 4;
    pub const HTTP_REQUEST_ERROR: i32 = 5;
}

fn main() {
    const URL: &str = "https://catbox.moe/user/api.php";
    const USER_AGENT: &str =
        "Mozilla/5.0 (X11; Linux x86_64; rv:123.0) Gecko/20100101 Firefox/123.0";
    // const THRESHOLD: usize = 2097152; // 2MiB
    const CONSTRAIN: u32 = 500; // Image height/width to always constrain to.
    const QUALITY: u8 = 80; // Image quality percentage

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

    let file_buffer: Vec<u8> = std::fs::read(&file_path).unwrap_or_else(|_| {
        eprintln!("Failed to read from filesystem");
        exit(error::FILE_SYSTEM_READ_ERROR);
    });

    let image_buffer: image::DynamicImage =
        image::load_from_memory(&file_buffer).unwrap_or_else(|_| {
            eprintln!("Failed to load image from memory");
            exit(error::IMAGE_LOADING_ERROR);
        });

    let resize_buffer: image::DynamicImage =
        if image_buffer.width() > CONSTRAIN || image_buffer.height() > CONSTRAIN {
            image_buffer.resize(CONSTRAIN, CONSTRAIN, image::imageops::FilterType::Nearest)
        } else {
            image_buffer
        };

    let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

    JpegEncoder::new_with_quality(&mut cursor, QUALITY)
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
        .append(format!("User-Agent: {}", USER_AGENT).as_str())
        .unwrap();

    let mut easy: Easy2<ResponseBody> = Easy2::new(ResponseBody(Vec::new()));
    easy.url(URL).unwrap();
    easy.http_headers(headers).unwrap();
    easy.httppost(form).unwrap();

    match easy.perform() {
        Ok(_) => {
            println!("{}", String::from_utf8_lossy(easy.get_ref().0.as_slice()));
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(error::HTTP_REQUEST_ERROR);
        }
    }
}
