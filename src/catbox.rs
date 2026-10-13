use anyhow::Result;
use log::{debug, info};
use reqwest::blocking::{
    Client,
    multipart::{Form, Part},
};
use std::time::Duration;

const CATBOX_URL: &str = "https://catbox.moe/user/api.php";

pub trait Service {
    fn upload(&self, data: Vec<u8>) -> Result<String>;
}

#[derive(Debug)]
pub struct Catbox {
    client: Client,
    user_hash: Option<String>,
}

impl Catbox {
    pub fn new(user_agent: &str, user_hash: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { client, user_hash })
    }
}

impl Service for Catbox {
    fn upload(&self, data: Vec<u8>) -> Result<String> {
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

        let response = self.client.post(CATBOX_URL).multipart(form).send()?;
        let status_code = response.status();
        let body = response.text()?;

        if !status_code.is_success() {
            anyhow::bail!("Catbox error: {}, {}", status_code, body);
        }

        info!("got link: \"{}\"", body);

        Ok(body)
    }
}
