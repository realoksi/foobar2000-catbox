use anyhow::{Result, bail};
use log::{debug, info};
use reqwest::blocking::{
    Client,
    multipart::{Form, Part},
};
use std::time::Duration;

const CATBOX_URL: &str = "https://catbox.moe/user/api.php";
const USER_AGENT: &str = concat!(
    "foobar2000-catbox/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/realoksi/foobar2000-catbox)"
);

pub trait Service {
    fn upload_image(&self, url: Option<String>, data: Vec<u8>) -> Result<String>;
}

#[derive(Debug)]
pub struct Catbox {
    client: Client,
    user_hash: Option<String>,
}

impl Catbox {
    pub fn new(user_agent: Option<&str>, user_hash: Option<String>, timeout: Option<u64>, connect_timeout: Option<u64>) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent.unwrap_or(USER_AGENT))
            .connect_timeout(Duration::from_secs(timeout.unwrap_or(10)))
            .timeout(Duration::from_secs(connect_timeout.unwrap_or(30)))
            .build()?;

        Ok(Self { client, user_hash })
    }
}

#[test]
fn test_catbox_new() {
    const USER_AGENT: &str = "foobar2000-catbox/test_catbox_new";
    let user_hash = "fa06e93babb0163ad24951b25".to_string(); // not a real hash

    let cb = Catbox::new(None, Some(user_hash.clone()), None, None).unwrap();

    assert_eq!(cb.user_hash, Some(user_hash));
}

impl Service for Catbox {
    fn upload_image(&self, url: Option<String>, data: Vec<u8>) -> Result<String> {
        let image_format = image::guess_format(&data)?;
        let mime_type = image_format.to_mime_type();
        let ext = image_format
            .extensions_str()
            .first()
            .copied()
            .unwrap_or_else(|| {
                // `image` might resolve a format without any registered extensions. fall back to
                // mime subtype, or just a generic bin.
                mime_type.strip_prefix("image/").unwrap_or("bin")
            });

        debug!(
            "image_format: {:?}, mime_type: {}, ext: {}",
            image_format, mime_type, ext
        );

        let mut form = Form::new().text("reqtype", "fileupload");

        form = form.part(
            "fileToUpload",
            Part::bytes(data)
                .file_name(format!("image.{}", ext))
                .mime_str(mime_type)?,
        );

        if let Some(user_hash) = &self.user_hash {
            debug!("adding user hash to form");
            form = form.text("userhash", user_hash.to_string());
        }

        info!("uploading multipart form");

        let response = self
            .client
            .post(url.unwrap_or(CATBOX_URL.to_string()))
            .multipart(form)
            .send()?;
        let status_code = response.status();
        let body = response.text()?;

        if !status_code.is_success() {
            bail!("Catbox error: {}, {}", status_code, body);
        }

        info!("received: \"{}\"", body);

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use image::{ColorType, DynamicImage};
    use std::io::Cursor;

    #[test]
    fn test_catbox_upload_image() {
        let mut server_guard = mockito::Server::new();

        let mock = server_guard
            .mock("POST", "/api.php")
            .with_status(200)
            .with_header("content-type", "text/html; charset=UTF-8")
            .with_body("https://files.catbox.moe/3e28av.jpg")
            .create();

        let url = server_guard.url();

        let mut cursor = Cursor::new(Vec::new());
        let rgba8 = DynamicImage::new(32, 32, ColorType::Rgb8);

        rgba8
            .write_with_encoder(PngEncoder::new_with_quality(
                &mut cursor,
                CompressionType::Best,
                FilterType::Adaptive,
            ))
            .unwrap();

        let cb = Catbox::new(None, None, None, None).unwrap();
        let response = cb
            .upload_image(
                Some(format!("{}/api.php", url).to_string()),
                cursor.into_inner(),
            )
            .unwrap();

        println!("Response: {:?}", response);

        mock.assert();
    }
}
