use std::{fs, io, path::Path, process::exit};

use curl::easy::{Easy2, Form, Handler, List, WriteError};

struct ResponseBody(Vec<u8>);

impl Handler for ResponseBody {
    // I stole this. Sorry
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

// TODO: Use the image crate to resize/compress an image when its size exceeds a specified threshold.

fn main() {
    const URL: &str = "https://catbox.moe/user/api.php";
    // const THRESHOLD: usize = 131072; // 128 KiB, see https://github.com/TheQwertiest/foo_discord_rich/pull/37#issuecomment-1464970437.

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

    let image_buffer = match fs::read(&file_path) {
        Ok(buffer) => buffer,
        Err(e) => {
            eprintln!("Failed to read file {}: {}", file_name, e);
            exit(-1);
        }
    };

    let mime_type = match image::guess_format(&image_buffer) {
        Ok(format) => {
            let extension = format.extensions_str().first().unwrap_or(&"");
            format!("image/{}", extension)
        }
        Err(_) => {
            eprintln!("Couldn't determine a mimetype type for this file.");
            exit(-1);
        }
    };

    let mut form = Form::new();
    form.part("reqtype").contents(b"fileupload").add().unwrap();
    form.part("userhash").contents(b"").add().unwrap();
    form.part("fileToUpload")
        .content_type(&mime_type)
        .buffer(&file_name, image_buffer)
        .add()
        .unwrap();

    let mut easy = Easy2::new(ResponseBody(Vec::new()));

    let mut headers = List::new();
    headers.append("Content-Type: multipart/form-data").unwrap();
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
