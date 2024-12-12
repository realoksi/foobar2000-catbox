use std::{
    env, fs,
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

mod settings;
use settings::Settings;

fn main() {
    let settings_path: &Path = &env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("settings.yml");

    let settings =
        Settings::from_str(&fs::read_to_string(settings_path).unwrap_or_default()).unwrap();

    if let Err(e) = settings.validate() {
        match e {
            settings::Error::EncodeFormatQualityOutOfRange(_) => {
                eprintln!(
                "\x1b[38;2;255;18;73mKey value out-of-bounds: encode_format_quality ({})\x1b[0m",
                settings.encode_format_quality
            );
                exit(1);
            }
            settings::Error::UnexpectedKeys => eprintln!(
                "\x1b[38;2;255;144;33mUnrecognized keys: {:?} (skipping)\x1b[0m",
                settings.unexpected.keys()
            ),
        }
    }

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

    let max_width: u32 = settings.resize_max_resolution.unwrap()[0].into();
    let max_height: u32 = settings.resize_max_resolution.unwrap()[1].into();

    let resize_buffer: image::DynamicImage =
        if image_buffer.width() > max_width || image_buffer.height() > max_height {
            image_buffer.resize(max_width, max_height, image::imageops::FilterType::Nearest)
        } else {
            image_buffer
        };

    let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

    JpegEncoder::new_with_quality(&mut cursor, settings.encode_format_quality)
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
        .append(
            format!(
                "User-Agent: {:?}",
                settings.unexpected.get("user_agent").unwrap()
            )
            .as_str(),
        )
        .unwrap();

    let mut easy: Easy2<ResponseBody> = Easy2::new(ResponseBody(Vec::new()));
    easy.url("https://catbox.moe/user/api.php").unwrap();
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
