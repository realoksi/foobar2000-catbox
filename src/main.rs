use std::{
    io::{self, Cursor},
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

fn main() {
    const URL: &str = "https://catbox.moe/user/api.php";
    const USER_AGENT: &str =
        "Mozilla/5.0 (X11; Linux x86_64; rv:123.0) Gecko/20100101 Firefox/123.0";
    // const THRESHOLD: usize = 2097152; // 2MiB
    const CONSTRAIN: u32 = 500; // Image height/width to always constrain to.
    const QUALITY: u8 = 80; // Image quality percentage

    let input = io::stdin()
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .trim()
        .to_string();

    let file_path = Path::new(&input);

    if !file_path.exists() {
        eprintln!("{:?} doesn't exist.", file_path.as_os_str());
        exit(-1);
    }

    let file_name = file_path.file_name().unwrap().to_str().unwrap();

    let file_buffer = match std::fs::read(&file_path) {
        Ok(buffer) => buffer,
        Err(e) => {
            eprintln!("Failed to read file {}: {}", file_name, e);
            exit(-1);
        }
    };

    let image_buffer = image::load_from_memory(&file_buffer).unwrap();

    let resize_buffer = if image_buffer.width() > CONSTRAIN || image_buffer.height() > CONSTRAIN {
        image_buffer.resize(CONSTRAIN, CONSTRAIN, image::imageops::FilterType::Nearest)
    } else {
        image_buffer
    };

    let mut cursor = Cursor::new(Vec::new());

    JpegEncoder::new_with_quality(&mut cursor, QUALITY)
        .encode_image(&resize_buffer)
        .unwrap();

    let mut form = Form::new();
    form.part("reqtype").contents(b"fileupload").add().unwrap();
    form.part("userhash").contents(b"").add().unwrap();
    form.part("fileToUpload")
        .content_type("image/jpeg")
        .buffer(&file_name, cursor.into_inner())
        .add()
        .unwrap();

    let mut easy = Easy2::new(ResponseBody(Vec::new()));

    let mut headers = List::new();
    headers.append("Content-Type: multipart/form-data").unwrap();
    headers
        .append(format!("User-Agent: {}", USER_AGENT).as_str())
        .unwrap();
    easy.url(URL).unwrap();
    easy.http_headers(headers).unwrap();
    easy.httppost(form).unwrap();

    match easy.perform() {
        Ok(_) => {
            println!("{}", String::from_utf8_lossy(easy.get_ref().0.as_slice()));
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(-1);
        }
    }
}
