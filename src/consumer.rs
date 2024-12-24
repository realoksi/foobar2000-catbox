use curl::{
    easy::{Easy2, Form, Handler, List, WriteError},
    Error,
};

pub trait Consumer {
    fn upload_image(&self, data: Vec<u8>) -> Result<String, Error>;
}

struct ResponseBody(Vec<u8>);

impl Handler for ResponseBody {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

fn perform(url: String, form: Form, user_agent: String) -> Result<String, Error> {
    let mut easy = Easy2::new(ResponseBody(Vec::new()));

    easy.url(url.as_str()).unwrap();
    easy.post(true).unwrap();
    easy.httppost(form).unwrap();

    let mut headers = List::new();

    headers
        .append(format!("User-Agent: {}", user_agent).as_str())
        .unwrap();

    easy.http_headers(headers).unwrap();
    easy.forbid_reuse(true).unwrap();

    match easy.perform() {
        Ok(_) => Ok(String::from_utf8_lossy(easy.get_ref().0.as_slice()).to_string()),
        Err(e) => Err(e),
    }
}

pub struct Catbox {
    user_agent: String,
}

impl Catbox {
    pub fn new(user_agent: Option<String>) -> Self {
        Self {
            user_agent: user_agent.unwrap_or(
                "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0".into(),
            ),
        }
    }
}

impl Consumer for Catbox {
    fn upload_image(&self, data: Vec<u8>) -> Result<String, Error> {
        let guessed_format = image::guess_format(&data).unwrap();
        let file_name = format!("image.{}", guessed_format.extensions_str()[0]);

        let mut form = Form::new();

        form.part("reqtype").contents(b"fileupload").add().unwrap();
        form.part("fileToUpload")
            .buffer(file_name.as_str(), data)
            .add()
            .unwrap();

        perform(
            "https://catbox.moe/user/api.php".into(),
            form,
            self.user_agent.clone(),
        )
    }
}

pub struct Litterbox {
    pub expire_time: String,
    pub user_agent: String,
}

impl Litterbox {
    pub fn new(expire_time: String, user_agent: Option<String>) -> Self {
        Self {
            expire_time,
            user_agent: user_agent.unwrap_or(
                "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0".into(),
            ),
        }
    }
}

impl Consumer for Litterbox {
    fn upload_image(&self, data: Vec<u8>) -> Result<String, Error> {
        let guessed_format = image::guess_format(&data).unwrap();
        let file_name = format!("image.{}", guessed_format.extensions_str()[0]);

        let mut form = Form::new();

        form.part("reqtype").contents(b"fileupload").add().unwrap();
        form.part("time")
            .contents(&self.expire_time.as_bytes())
            .add()
            .unwrap();
        form.part("fileToUpload")
            .buffer(file_name.as_str(), data)
            .add()
            .unwrap();

        perform(
            "https://litterbox.catbox.moe/resources/internals/api.php".into(),
            form,
            self.user_agent.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::ExpireTime;

    use super::*;
    use base64::prelude::*;

    #[test]
    fn test_consumers() {
        let sample_image = BASE64_STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQAAAAA3bvkkAAAACklEQVR4AWNgAAAAAgABc3UBGAAAAABJRU5ErkJggg==").unwrap().to_vec();

        assert!(Catbox::new(None).upload_image(sample_image.clone()).is_ok());

        assert!(Litterbox::new(ExpireTime::default().to_string(), None)
            .upload_image(sample_image.clone())
            .is_ok());
    }

    #[test]
    fn test_perform() {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0".to_string();

        assert!(perform(
            "https://example.com/".into(),
            Form::new(),
            user_agent.clone(),
        )
        .is_ok());

        assert!(perform(
            "http://localhost/".into(),
            Form::new(),
            user_agent.clone(),
        )
        .is_err());
    }
}
